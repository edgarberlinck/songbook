use serde::Serialize;

use crate::domain::song::{RenderedLine, Song};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongSummaryDto {
    pub id: String,
    pub title: String,
    pub artist: Option<String>,
    pub key: Option<String>,
    pub favorite: bool,
    pub last_modified: u64,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SongDto {
    pub id: String,
    pub title: String,
    pub subtitle: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub key: Option<String>,
    pub capo: Option<i32>,
    pub tempo: Option<i32>,
    pub tags: Vec<String>,
    pub notes: Option<String>,
    pub favorite: bool,
    pub created_at: u64,
    pub last_modified: u64,
    pub content: String,
    pub preview: Vec<RenderedLine>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryDto {
    pub songs: Vec<SongSummaryDto>,
    pub available_tags: Vec<String>,
}

impl From<&Song> for SongSummaryDto {
    fn from(value: &Song) -> Self {
        Self {
            id: value.id.clone(),
            title: value.title.clone(),
            artist: value.artist.clone(),
            key: value.key.clone(),
            favorite: value.favorite,
            last_modified: value.last_modified,
            tags: value.tags.clone(),
        }
    }
}

impl From<Song> for SongDto {
    fn from(value: Song) -> Self {
        Self {
            id: value.id,
            title: value.title,
            subtitle: value.subtitle,
            artist: value.artist,
            album: value.album,
            key: value.key,
            capo: value.capo,
            tempo: value.tempo,
            tags: value.tags,
            notes: value.notes,
            favorite: value.favorite,
            created_at: value.created_at,
            last_modified: value.last_modified,
            content: value.content,
            preview: value.preview,
        }
    }
}
