use std::sync::Mutex;

mod ui;

pub fn run() {
    tauri::Builder::default()
        .manage(Mutex::new(ui::commands::AppState::default()))
        .invoke_handler(tauri::generate_handler![
            ui::commands::load_library,
            ui::commands::load_song,
            ui::commands::save_song,
            ui::commands::preview_song,
            ui::commands::transpose_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Songbook")
}
