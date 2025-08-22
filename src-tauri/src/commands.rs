// Implement core SwitchHosts commands for Tauri
// NOTE: Returned JSON shapes aim to match SwitchHosts TypeScript interfaces in
// SwitchHosts/src/common/data.d.ts (IHostsListObject, IHostsContentObject, ITrashcanObject, etc.)
// We keep storage as serde_json::Value for flexibility but preserve fields like
// `id`, `title`, `on`, `type`, `children`, `content`, `add_time_ms` to maintain compatibility.
use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn data_dir() -> PathBuf {
    if let Ok(dir) = env::var("SWEETHOSTS_DATA_DIR") {
        return PathBuf::from(dir);
    }

    if cfg!(target_os = "macos") || cfg!(target_os = "linux") {
        if let Ok(home) = env::var("HOME") {
            return PathBuf::from(home).join(".sweethosts");
        }
    }

    // fallback to current dir
    PathBuf::from(".").join("sweethosts")
}

fn ensure_data_dir() -> std::io::Result<()> {
    let d = data_dir();
    if !d.exists() {
        fs::create_dir_all(&d)?;
    }
    Ok(())
}

fn read_json_array(p: PathBuf) -> Vec<Value> {
    if !p.exists() {
        return vec![];
    }
    match fs::read_to_string(p) {
        Ok(s) => serde_json::from_str(&s).unwrap_or_default(),
        Err(_) => vec![],
    }
}

fn write_json_array(p: PathBuf, v: &Vec<Value>) -> bool {
    match fs::write(p, serde_json::to_string(v).unwrap_or_default()) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
pub fn get_list() -> Vec<Value> {
    if let Err(_) = ensure_data_dir() {
        return vec![];
    }

    let mut p = data_dir();
    p.push("list.json");
    read_json_array(p)
}

#[tauri::command]
pub fn set_list(v: Vec<Value>) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("list.json");
    write_json_array(p, &v)
}

#[tauri::command]
pub fn get_content_of_list() -> String {
    // read list.json and collect ids where on == true
    let list = get_list();
    let mut ids: Vec<String> = Vec::new();

    fn collect(items: &Vec<Value>, out: &mut Vec<String>) {
        for item in items {
            if let Some(on) = item.get("on").and_then(|v| v.as_bool()) {
                if on {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        out.push(id.to_string());
                    }
                }
            }
            if let Some(children) = item.get("children").and_then(|v| v.as_array()) {
                collect(children, out);
            }
        }
    }

    collect(&list, &mut ids);

    let mut contents: Vec<String> = Vec::new();
    for id in ids {
        let mut p = data_dir();
        p.push(format!("hosts_content_{}.txt", id));
        if let Ok(s) = fs::read_to_string(p) {
            contents.push(format!("# file: {}\n{}", id, s));
        }
    }

    let content = contents.join("\n\n");
    content
}

#[tauri::command]
pub fn get_item_from_list(id: String) -> Option<Value> {
    let list = get_list();
    for item in list {
        if let Some(item_id) = item.get("id") {
            if item_id == &Value::String(id.clone()) {
                return Some(item);
            }
        }
    }
    None
}

#[tauri::command]
pub fn move_to_trashcan(id: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }

    let mut list_path = data_dir();
    list_path.push("list.json");
    let mut list = read_json_array(list_path.clone());

    // find and remove
    let mut removed: Option<Value> = None;
    list.retain(|item| {
        if let Some(item_id) = item.get("id") {
            if item_id == &Value::String(id.clone()) {
                removed = Some(item.clone());
                return false;
            }
        }
        true
    });

    // save updated list
    write_json_array(list_path, &list);

    if let Some(mut obj) = removed {
        // set on = false
        if let Some(map) = obj.as_object_mut() {
            map.insert("on".to_string(), Value::Bool(false));
        }

        let mut trash_path = data_dir();
        trash_path.push("trashcan.json");
        let mut trash = read_json_array(trash_path.clone());

        let mut entry = serde_json::Map::new();
        entry.insert("data".to_string(), obj);
        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0) as i64;
        entry.insert(
            "add_time_ms".to_string(),
            Value::Number(serde_json::Number::from(ms)),
        );
        entry.insert("parent_id".to_string(), Value::Null);

        trash.push(Value::Object(entry));
        write_json_array(trash_path, &trash);
    }

    true
}

#[tauri::command]
pub fn move_many_to_trashcan(ids: Vec<String>) -> bool {
    for id in ids {
        let _ = move_to_trashcan(id);
    }
    true
}

#[tauri::command]
pub fn get_trashcan_list() -> Vec<Value> {
    if let Err(_) = ensure_data_dir() {
        return vec![];
    }
    let mut p = data_dir();
    p.push("trashcan.json");
    read_json_array(p)
}

#[tauri::command]
pub fn clear_trashcan() -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("trashcan.json");
    match fs::write(p, "[]") {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn delete_item_from_trashcan(id: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("trashcan.json");
    let mut trash = read_json_array(p.clone());
    let original_len = trash.len();
    trash.retain(|item| {
        if let Some(d) = item.get("data") {
            if let Some(item_id) = d.get("id") {
                return item_id != &Value::String(id.clone());
            }
        }
        true
    });
    let changed = trash.len() != original_len;
    write_json_array(p, &trash);
    changed
}

#[tauri::command]
pub fn restore_item_from_trashcan(id: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut trash_path = data_dir();
    trash_path.push("trashcan.json");
    let mut trash = read_json_array(trash_path.clone());

    let mut restored: Option<Value> = None;
    trash.retain(|item| {
        if let Some(d) = item.get("data") {
            if let Some(item_id) = d.get("id") {
                if item_id == &Value::String(id.clone()) {
                    restored = Some(d.clone());
                    return false;
                }
            }
        }
        true
    });

    if let Some(obj) = restored {
        // append back to list
        let mut list_path = data_dir();
        list_path.push("list.json");
        let mut list = read_json_array(list_path.clone());
        list.push(obj);
        write_json_array(list_path, &list);

        // save updated trash
        write_json_array(trash_path, &trash);
        return true;
    }

    false
}

#[tauri::command]
pub fn config_get(key: String) -> Option<Value> {
    if let Err(_) = ensure_data_dir() {
        return None;
    }
    let mut p = data_dir();
    p.push("config.json");
    if !p.exists() {
        return None;
    }
    match fs::read_to_string(p) {
        Ok(s) => match serde_json::from_str::<Value>(&s) {
            Ok(v) => v.get(&key).cloned(),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

#[tauri::command]
pub fn config_set(key: String, val: Value) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("config.json");
    let mut obj = if p.exists() {
        match fs::read_to_string(&p) {
            Ok(s) => {
                serde_json::from_str::<Value>(&s).unwrap_or(Value::Object(serde_json::Map::new()))
            }
            Err(_) => Value::Object(serde_json::Map::new()),
        }
    } else {
        Value::Object(serde_json::Map::new())
    };

    if let Some(map) = obj.as_object_mut() {
        map.insert(key, val);
    }

    match fs::write(
        p,
        serde_json::to_string(&obj).unwrap_or_else(|_| "{}".to_string()),
    ) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn show_item_in_folder(link: String) -> bool {
    // If it's a directory, open it; otherwise open parent folder.
    let p = std::path::Path::new(&link);
    if p.exists() && p.is_dir() {
        return open::that(p).is_ok();
    }
    if let Some(parent) = p.parent() {
        return open::that(parent).is_ok();
    }
    false
}

#[tauri::command]
pub fn get_data_dir() -> String {
    data_dir().to_string_lossy().into_owned()
}

#[tauri::command]
pub fn get_default_data_dir() -> String {
    // Mirror behavior of data_dir for now
    data_dir().to_string_lossy().into_owned()
}

#[tauri::command]
pub fn get_basic_data() -> String {
    #[derive(Serialize)]
    struct BasicData {
        list: Vec<Value>,
        trashcan: Vec<Value>,
        version: Value,
    }

    let list = get_list();
    let trash = get_trashcan_list();
    let bd = BasicData {
        list,
        trashcan: trash,
        version: Value::String("0.0.0".to_string()),
    };
    serde_json::to_string(&bd).unwrap_or_else(|_| "{}".to_string())
}

// Hosts path
#[tauri::command]
pub fn get_path_of_system_hosts() -> String {
    if cfg!(target_os = "windows") {
        std::env::var("windir")
            .map(|w| format!("{}\\system32\\drivers\\etc\\hosts", w))
            .unwrap_or_else(|_| "C:\\Windows\\system32\\drivers\\etc\\hosts".to_string())
    } else {
        "/etc/hosts".to_string()
    }
}

#[tauri::command]
pub fn get_system_hosts() -> String {
    let p = get_path_of_system_hosts();
    fs::read_to_string(p).unwrap_or_default()
}

#[tauri::command]
pub fn get_hosts_content(id: String) -> String {
    if let Err(_) = ensure_data_dir() {
        return String::new();
    }
    let mut p = data_dir();
    p.push(format!("hosts_content_{}.txt", id));
    match fs::read_to_string(p) {
        Ok(s) => s,
        Err(_) => String::new(),
    }
}

#[tauri::command]
pub fn set_hosts_content(id: String, content: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push(format!("hosts_content_{}.txt", id));
    fs::write(p, content).is_ok()
}

#[tauri::command]
pub fn set_system_hosts(content: String, opts: Option<String>) -> Value {
    let sys_path = get_path_of_system_hosts();

    // read old content
    let old_content = fs::read_to_string(&sys_path).unwrap_or_default();

    // respect safe mode
    if std::env::var("SWEETHOSTS_SAFE_MODE").unwrap_or_default() == "1" {
        // write to temp file instead
        let mut tmp = env::temp_dir();
        tmp.push(format!(
            "sweethosts_safe_{}.hosts",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        let _ = fs::write(&tmp, &content);
        return json!({ "success": true, "old_content": old_content, "new_content": content, "safe_path": tmp.to_string_lossy() });
    }

    // try direct write first
    match fs::write(&sys_path, &content) {
        Ok(_) => {
            // success
            let res =
                json!({ "success": true, "old_content": old_content, "new_content": content });
            // add history entries: old and new
            let _ = add_history_internal(&old_content);
            let _ = add_history_internal(&content);
            return res;
        }
        Err(_) => {
            // try sudo fallback on unix
            if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
                if let Some(pw) = opts {
                    // write tmp file
                    let mut tmp = env::temp_dir();
                    let rand_part = format!(
                        "{}",
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .map(|d| d.as_nanos())
                            .unwrap_or(0)
                    );
                    tmp.push(format!("swh_{}.txt", rand_part));
                    let _ = fs::write(&tmp, &content);

                    let cmd = format!(
                        "echo '{}' | sudo -S sh -c 'cat \"{}\" > \"{}\" && chmod 644 \"{}\"'",
                        pw.replace("'", "'\\''"),
                        tmp.to_string_lossy(),
                        sys_path,
                        sys_path
                    );

                    let output = Command::new("sh").arg("-c").arg(cmd).output();
                    // cleanup tmp
                    let _ = fs::remove_file(&tmp);

                    if let Ok(o) = output {
                        if o.status.success() {
                            let res = json!({ "success": true, "old_content": old_content, "new_content": content });
                            let _ = add_history_internal(&old_content);
                            let _ = add_history_internal(&content);
                            return res;
                        } else {
                            let msg = String::from_utf8_lossy(&o.stderr).to_string();
                            return json!({ "success": false, "code": "no_access", "message": msg });
                        }
                    }
                }
            }
        }
    }

    json!({ "success": false, "code": "no_access" })
}

// Internal helper for history: append raw content to history.json with id and timestamp
fn add_history_internal(content: &str) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("history.json");
    let mut history = read_json_array(p.clone());

    let mut entry = serde_json::Map::new();
    let id_str = format!(
        "{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    entry.insert("id".to_string(), Value::String(id_str));
    entry.insert("content".to_string(), Value::String(content.to_string()));
    let ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0) as i64;
    entry.insert(
        "add_time_ms".to_string(),
        Value::Number(serde_json::Number::from(ms)),
    );

    history.push(Value::Object(entry));

    write_json_array(p, &history)
}

#[tauri::command]
pub fn get_history_list() -> Vec<Value> {
    if let Err(_) = ensure_data_dir() {
        return vec![];
    }
    let mut p = data_dir();
    p.push("history.json");
    read_json_array(p)
}

#[tauri::command]
pub fn delete_history(id: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("history.json");
    let mut history = read_json_array(p.clone());
    let original = history.len();
    history.retain(|item| item.get("id") != Some(&Value::String(id.clone())));
    let changed = history.len() != original;
    write_json_array(p, &history);
    changed
}

#[tauri::command]
pub fn refresh_hosts() -> Value {
    // read list.json and collect ids where on == true
    let list = get_list();
    let mut ids: Vec<String> = Vec::new();

    fn collect(items: &Vec<Value>, out: &mut Vec<String>) {
        for item in items {
            if let Some(on) = item.get("on").and_then(|v| v.as_bool()) {
                if on {
                    if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                        out.push(id.to_string());
                    }
                }
            }
            if let Some(children) = item.get("children").and_then(|v| v.as_array()) {
                collect(children, out);
            }
        }
    }

    collect(&list, &mut ids);

    let mut contents: Vec<String> = Vec::new();
    for id in ids {
        let mut p = data_dir();
        p.push(format!("hosts_content_{}.txt", id));
        if let Ok(s) = fs::read_to_string(p) {
            contents.push(format!("# file: {}\n{}", id, s));
        }
    }

    let content = contents.join("\n\n");
    // call set_system_hosts with no sudo
    set_system_hosts(content, None)
}

#[tauri::command]
pub fn update_tray_title(_title: String) -> bool {
    // Tray handled in UI layer; noop for now
    true
}

#[tauri::command]
pub fn migrate_check() -> bool {
    // For now, no migration
    false
}

#[tauri::command]
pub fn migrate_data() -> bool {
    // Placeholder: nothing to do
    true
}

#[tauri::command]
pub fn export_data() -> Option<String> {
    // export data.json to project data dir (ask user in UI normally)
    if let Err(_) = ensure_data_dir() {
        return None;
    }
    let mut p = data_dir();
    p.push("exported_data.json");

    // collect data
    let list = get_list();
    let trash = get_trashcan_list();
    let history = get_history_list();

    let obj = json!({ "list": list, "trashcan": trash, "history": history, "version": [0,0,0,0] });
    if fs::write(&p, serde_json::to_string(&obj).unwrap_or_default()).is_ok() {
        return p.to_str().map(|s| s.to_string());
    }
    None
}

// cmd history (command run history) — stored in cmd_history.json
#[tauri::command]
pub fn cmd_get_history_list() -> Vec<Value> {
    if let Err(_) = ensure_data_dir() {
        return vec![];
    }
    let mut p = data_dir();
    p.push("cmd_history.json");
    read_json_array(p)
}

#[tauri::command]
pub fn cmd_delete_history(id: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("cmd_history.json");
    let mut v = read_json_array(p.clone());
    let original = v.len();
    v.retain(|item| {
        // support both "id" and "_id"
        let keep_id = item
            .get("id")
            .and_then(|x| x.as_str())
            .map(|s| s != id)
            .unwrap_or(true);
        let keep_id2 = item
            .get("_id")
            .and_then(|x| x.as_str())
            .map(|s| s != id)
            .unwrap_or(true);
        keep_id && keep_id2
    });
    let changed = v.len() != original;
    write_json_array(p, &v);
    changed
}

#[tauri::command]
pub fn cmd_clear_history() -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("cmd_history.json");
    match fs::write(p, "[]") {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn try_to_run() -> Option<Value> {
    // read command from config key 'cmd_after_hosts_apply'
    if let Some(cmd_val) = config_get("cmd_after_hosts_apply".to_string()) {
        if let Some(cmd) = cmd_val.as_str() {
            if cmd.trim().is_empty() {
                return None;
            }
            // run command via shell
            let output = Command::new("sh").arg("-c").arg(cmd).output();
            let (success, stdout_s, stderr_s) = match output {
                Ok(o) => (
                    o.status.success(),
                    String::from_utf8_lossy(&o.stdout).to_string(),
                    String::from_utf8_lossy(&o.stderr).to_string(),
                ),
                Err(e) => (false, "".to_string(), format!("{}", e)),
            };

            let mut entry = serde_json::Map::new();
            let id_str = format!(
                "{}",
                SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_millis())
                    .unwrap_or(0)
            );
            entry.insert("id".to_string(), Value::String(id_str));
            entry.insert("success".to_string(), Value::Bool(success));
            entry.insert("stdout".to_string(), Value::String(stdout_s.clone()));
            entry.insert("stderr".to_string(), Value::String(stderr_s.clone()));
            let ms = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0) as i64;
            entry.insert(
                "add_time_ms".to_string(),
                Value::Number(serde_json::Number::from(ms)),
            );

            // append to cmd_history.json
            if let Err(_) = ensure_data_dir() {
                return Some(Value::Object(entry));
            }
            let mut p = data_dir();
            p.push("cmd_history.json");
            let mut v = read_json_array(p.clone());
            v.push(Value::Object(entry.clone()));
            let _ = write_json_array(p, &v);

            return Some(Value::Object(entry));
        }
    }
    None
}

#[tauri::command]
pub fn check_update() -> Option<String> {
    // lightweight stub: no remote check implemented yet
    None
}

#[tauri::command]
pub fn import_data(path: String) -> Result<bool, String> {
    // read file and load into data dir
    match fs::read_to_string(path) {
        Ok(s) => {
            match serde_json::from_str::<Value>(&s) {
                Ok(v) => {
                    // naive import: write list and trash
                    if let Some(list) = v.get("list").and_then(|x| x.as_array()) {
                        let mut p = data_dir();
                        p.push("list.json");
                        write_json_array(p, list);
                    }
                    if let Some(trash) = v.get("trashcan").and_then(|x| x.as_array()) {
                        let mut p = data_dir();
                        p.push("trashcan.json");
                        write_json_array(p, trash);
                    }
                    return Ok(true);
                }
                Err(e) => return Err(format!("parse_error: {}", e)),
            }
        }
        Err(e) => return Err(format!("read_error: {}", e)),
    }
}

#[tauri::command]
pub fn import_data_from_url(url: String) -> Result<bool, String> {
    let client = Client::new();
    match client.get(&url).send() {
        Ok(mut r) => {
            if !r.status().is_success() {
                return Err(format!("error_{}", r.status().as_u16()));
            }
            let mut s = String::new();
            if r.read_to_string(&mut s).is_err() {
                return Err("read_error".to_string());
            }
            match serde_json::from_str::<Value>(&s) {
                Ok(v) => {
                    if let Some(list) = v.get("list").and_then(|x| x.as_array()) {
                        let mut p = data_dir();
                        p.push("list.json");
                        write_json_array(p, list);
                    }
                    return Ok(true);
                }
                Err(e) => Err(format!("parse_error: {}", e)),
            }
        }
        Err(e) => Err(format!("request_error: {}", e)),
    }
}

// find history commands
#[tauri::command]
pub fn find_get_history() -> Vec<Value> {
    // reuse cfg-like storage: use file find_history.json
    if let Err(_) = ensure_data_dir() {
        return vec![];
    }
    let mut p = data_dir();
    p.push("find_history.json");
    read_json_array(p)
}

#[tauri::command]
pub fn find_set_history(list: Vec<Value>) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("find_history.json");
    write_json_array(p, &list)
}

#[tauri::command]
pub fn find_add_history(item: Value) -> Vec<Value> {
    let mut a = find_get_history();
    a.retain(|v| v != &item);
    a.push(item);
    let _ = find_set_history(a.clone());
    a
}

#[tauri::command]
pub fn config_all() -> String {
    if let Err(_) = ensure_data_dir() {
        return "{}".to_string();
    }

    let mut p = data_dir();
    p.push("config.json");

    // default configs based on SwitchHosts/src/common/default_configs.ts
    let mut defaults = json!({
        "left_panel_show": true,
        "left_panel_width": 270,
        "use_system_window_frame": false,
        "write_mode": "append",
        "history_limit": 50,
        "locale": Value::Null,
        "theme": "light",
        "choice_mode": 2,
        "show_title_on_tray": false,
        "hide_at_launch": false,
        "send_usage_data": false,
        "cmd_after_hosts_apply": "",
        "remove_duplicate_records": false,
        "hide_dock_icon": false,
        "use_proxy": false,
        "proxy_protocol": "http",
        "proxy_host": "",
        "proxy_port": 0,
        "http_api_on": false,
        "http_api_only_local": true,
        "tray_mini_window": true,
        "multi_chose_folder_switch_all": false,
        "auto_download_update": true,
        "env": "PROD"
    });

    if p.exists() {
        if let Ok(s) = fs::read_to_string(&p) {
            if let Ok(user_cfg) = serde_json::from_str::<Value>(&s) {
                if let Some(user_map) = user_cfg.as_object() {
                    if let Some(def_map) = defaults.as_object_mut() {
                        for (k, v) in user_map.iter() {
                            def_map.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
        }
    }

    serde_json::to_string(&defaults).unwrap_or_else(|_| "{}".to_string())
}

#[tauri::command]
pub fn config_update(cfg: String) -> bool {
    if let Err(_) = ensure_data_dir() {
        return false;
    }
    let mut p = data_dir();
    p.push("config.json");
    fs::write(p, cfg).is_ok()
}

#[tauri::command]
pub fn close_main_window() -> bool {
    // Window control should be handled via tauri::Window in app code; noop
    true
}

#[tauri::command]
pub fn quit() -> bool {
    // Tauri app exit should be invoked from main; noop
    true
}

#[tauri::command]
pub fn open_url(url: String) -> bool {
    // Use open crate to open external URLs
    match open::that(url) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tauri::command]
pub fn noop() -> String {
    "noop".to_string()
}
