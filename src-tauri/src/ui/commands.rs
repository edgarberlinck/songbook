use std::{path::PathBuf, sync::Mutex};

use songbook_core::application::{
    dto::{LibraryDto, SongDto},
    song_service::SongService,
};
use tauri::State;

pub struct AppState {
    root: PathBuf,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

#[tauri::command]
pub fn load_library(state: State<'_, Mutex<AppState>>) -> Result<LibraryDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.load_library()
}

#[tauri::command]
pub fn load_song(id: String, state: State<'_, Mutex<AppState>>) -> Result<SongDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.load_song(&id)
}

#[tauri::command]
pub fn save_song(id: String, content: String, state: State<'_, Mutex<AppState>>) -> Result<SongDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.save_song(&id, &content)
}

#[tauri::command]
pub fn preview_song(id: String, content: String, state: State<'_, Mutex<AppState>>) -> Result<SongDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.preview_song(&id, &content)
}

#[tauri::command]
pub fn transpose_content(content: String, semitones: i32) -> Result<String, String> {
    let root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    Ok(SongService::new(root)?.transpose_content(&content, semitones))
}
