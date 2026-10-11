mod commands;
pub mod core;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(commands::GameState::new())
        .invoke_handler(tauri::generate_handler![
            commands::new_game,
            commands::get_state,
            commands::send_input,
            commands::tick,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
