// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
pub mod commands;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            commands::ping,
            commands::get_list,
            commands::set_list,
            commands::get_basic_data,
            commands::get_content_of_list,
            commands::get_system_hosts,
            commands::get_hosts_content,
            commands::set_hosts_content,
            commands::set_system_hosts,
            commands::update_tray_title,
            commands::migrate_check,
            commands::migrate_data,
            commands::config_all,
            commands::config_update,
            commands::get_item_from_list,
            commands::move_to_trashcan,
            commands::move_many_to_trashcan,
            commands::get_trashcan_list,
            commands::clear_trashcan,
            commands::delete_item_from_trashcan,
            commands::restore_item_from_trashcan,
            commands::config_get,
            commands::config_set,
            commands::show_item_in_folder,
            commands::get_data_dir,
            commands::get_default_data_dir,
            commands::get_history_list,
            commands::delete_history,
            commands::refresh_hosts,
            commands::close_main_window,
            commands::quit,
            commands::open_url,
            // cmd & misc
            commands::cmd_get_history_list,
            commands::cmd_delete_history,
            commands::cmd_clear_history,
            commands::try_to_run,
            commands::check_update,
            commands::noop
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
