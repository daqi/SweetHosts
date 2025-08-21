use std::env;
fn main() {
    // ensure safe mode
    env::set_var("SWEETHOSTS_SAFE_MODE", "1");

    println!("Running smoke tests in SAFE MODE");

    // ping
    let p = sweethosts_lib::commands::ping();
    println!("ping -> {}", p);

    // set and get list
    let list = vec![serde_json::json!({"id": "a1", "title": "test", "on": true})];
    let ok = sweethosts_lib::commands::set_list(list.clone());
    println!("set_list -> {}", ok);
    let got = sweethosts_lib::commands::get_list();
    println!("get_list -> {} items", got.len());

    // move to trashcan
    let _ = sweethosts_lib::commands::move_to_trashcan("a1".to_string());
    let trash = sweethosts_lib::commands::get_trashcan_list();
    println!("trash list -> {} items", trash.len());

    // refresh hosts (should not write system hosts in safe mode)
    let res = sweethosts_lib::commands::refresh_hosts();
    println!("refresh_hosts -> {}", res);

    // get_basic_data
    let bd = sweethosts_lib::commands::get_basic_data();
    println!("get_basic_data -> {}", bd);

    // history
    let hist = sweethosts_lib::commands::get_history_list();
    println!("history items -> {}", hist.len());
}
