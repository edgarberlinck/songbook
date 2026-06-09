use std::{collections::BTreeSet, path::PathBuf};

use crate::{
    application::dto::{LibraryDto, SongDto, SongSummaryDto},
    domain::{chordpro::parse_song, song::Song, transpose::transpose_bracketed_line},
    infrastructure::{
        file_song_repository::FileSongRepository, sqlite_index::SqliteIndex, sync::{CloudKitProvider, FolderProvider, GitProvider, OneDriveProvider},
    },
};

pub struct SongService {
    repository: FileSongRepository,
    index: SqliteIndex,
}

impl SongService {
    pub fn new(root: PathBuf) -> Result<Self, String> {
        let repository = FileSongRepository::new(root.join("songs"));
        let index = SqliteIndex::new(root.join("songbook.sqlite3")).map_err(|error| error.to_string())?;
        let _ = (CloudKitProvider, OneDriveProvider, FolderProvider, GitProvider);

        Ok(Self { repository, index })
    }

    pub fn load_library(&self) -> Result<LibraryDto, String> {
        let songs = self.reindex()?;
        let available_tags = songs
            .iter()
            .flat_map(|song| song.tags.iter().cloned())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();

        Ok(LibraryDto {
            songs: songs.iter().map(SongSummaryDto::from).collect(),
            available_tags,
        })
    }

    pub fn load_song(&self, id: &str) -> Result<SongDto, String> {
        let songs = self.reindex()?;
        let song = songs
            .into_iter()
            .find(|song| song.id == id)
            .ok_or_else(|| format!("Song with id '{id}' was not found"))?;
        Ok(song.into())
    }

    pub fn save_song(&self, id: &str, content: &str) -> Result<SongDto, String> {
        let songs = self.repository.read_all().map_err(|error| error.to_string())?;
        let existing = songs
            .into_iter()
            .find(|song| song.id == id)
            .ok_or_else(|| format!("Song with id '{id}' was not found"))?;

        self.repository
            .write_song(&existing.path, content)
            .map_err(|error| error.to_string())?;

        let refreshed = self
            .repository
            .read_song(&existing.path, parse_song)
            .map_err(|error| error.to_string())?;

        self.index.upsert_song(&refreshed).map_err(|error| error.to_string())?;
        Ok(refreshed.into())
    }

    pub fn preview_song(&self, id: &str, content: &str) -> Result<SongDto, String> {
        let songs = self.repository.read_all().map_err(|error| error.to_string())?;
        let existing = songs
            .into_iter()
            .find(|song| song.id == id)
            .ok_or_else(|| format!("Song with id '{id}' was not found"))?;

        Ok(parse_song(
            &existing.path,
            content.to_string(),
            existing.created_at,
            existing.last_modified,
        )
        .into())
    }

    pub fn transpose_content(&self, content: &str, semitones: i32) -> String {
        content
            .lines()
            .map(|line| transpose_bracketed_line(line, semitones))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn reindex(&self) -> Result<Vec<Song>, String> {
        let songs = self.repository.read_all().map_err(|error| error.to_string())?;
        self.index.reset().map_err(|error| error.to_string())?;
        for song in &songs {
            self.index.upsert_song(song).map_err(|error| error.to_string())?;
        }
        Ok(songs)
    }
}
