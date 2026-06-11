use std::{collections::BTreeSet, path::PathBuf, time::SystemTime};

use crate::{
    application::dto::{LibraryDto, SongDto, SongSummaryDto},
    domain::{chordpro::parse_song, song::Song, transpose::transpose_bracketed_line},
    infrastructure::{
        cifraclub_importer::CifraClubImporter, file_song_repository::FileSongRepository, sqlite_index::SqliteIndex, sync::{CloudKitProvider, FolderProvider, GitProvider, OneDriveProvider},
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

    pub fn create_song(&self) -> Result<SongDto, String> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis();
        let file_name = format!("new-song-{timestamp}.chordpro");
        let path = self.repository.songs_dir().join(file_name);
        let content = "{title: New Song}\n\n";

        self.repository
            .write_song(&path.to_string_lossy(), content)
            .map_err(|error| error.to_string())?;

        let created = self
            .repository
            .read_song(&path, parse_song)
            .map_err(|error| error.to_string())?;

        self.index.upsert_song(&created).map_err(|error| error.to_string())?;
        Ok(created.into())
    }

    pub fn import_song_from_url(&self, url: &str) -> Result<SongDto, String> {
        let imported = CifraClubImporter::import_from_url(url)?;
        std::fs::create_dir_all(self.repository.songs_dir()).map_err(|error| error.to_string())?;

        let safe_name = sanitize_filename(&imported.title);
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis();
        let path = self
            .repository
            .songs_dir()
            .join(format!("{}-{}.chordpro", safe_name, timestamp));

        let mut content = String::new();
        content.push_str(&format!("{{title: {}}}\n", imported.title));
        if let Some(artist) = imported.artist.as_ref() {
            content.push_str(&format!("{{artist: {artist}}}\n"));
        }
        if let Some(song_key) = imported.key.as_ref() {
            content.push_str(&format!("{{key: {song_key}}}\n"));
        }
        if let Some(capo) = imported.capo {
            content.push_str(&format!("{{capo: {capo}}}\n"));
        }
        if let Some(tuning) = imported.tuning.as_ref() {
            content.push_str(&format!("{{notes: Tuning: {tuning}}}\n"));
        }
        content.push('\n');
        content.push_str(imported.body.trim());
        content.push('\n');

        self.repository
            .write_song(&path.to_string_lossy(), &content)
            .map_err(|error| error.to_string())?;

        let created = self
            .repository
            .read_song(&path, parse_song)
            .map_err(|error| error.to_string())?;
        self.index.upsert_song(&created).map_err(|error| error.to_string())?;
        Ok(created.into())
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

fn sanitize_filename(value: &str) -> String {
    let mut sanitized = String::new();
    let mut previous_dash = false;

    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            sanitized.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            sanitized.push('-');
            previous_dash = true;
        }
    }

    let trimmed = sanitized.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "imported-song".to_string()
    } else {
        trimmed
    }
}
