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
            root: workspace_root(),
        }
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    match manifest_dir.parent() {
        Some(parent) => parent.to_path_buf(),
        None => manifest_dir,
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
pub fn delete_song(id: String, state: State<'_, Mutex<AppState>>) -> Result<(), String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.delete_song(&id)
}

#[tauri::command]
pub fn preview_song(id: String, content: String, state: State<'_, Mutex<AppState>>) -> Result<SongDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.preview_song(&id, &content)
}

#[tauri::command]
pub fn create_song(state: State<'_, Mutex<AppState>>) -> Result<SongDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.create_song()
}

#[tauri::command]
pub fn import_song_from_url(url: String, state: State<'_, Mutex<AppState>>) -> Result<SongDto, String> {
    let guard = state.lock().map_err(|_| "Failed to access application state".to_string())?;
    SongService::new(guard.root.clone())?.import_song_from_url(&url)
}

#[tauri::command]
pub fn transpose_content(content: String, semitones: i32) -> Result<String, String> {
    Ok(SongService::new(workspace_root())?.transpose_content(&content, semitones))
}
