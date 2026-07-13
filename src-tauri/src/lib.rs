mod commands;
pub mod models;
pub mod scanner;
pub mod scoring;
pub mod snapshot;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            commands::get_player_card,
            commands::save_png_file,
            commands::list_available_weeks,
            commands::get_stats_for_range
        ])
        .run(tauri::generate_context!())
        .expect("error while running ViberCard");
}
