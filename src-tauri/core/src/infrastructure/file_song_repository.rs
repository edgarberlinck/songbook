use std::{fs, io, path::{Path, PathBuf}, time::SystemTime};

use crate::domain::{chordpro::parse_song, song::Song};

pub struct FileSongRepository {
    songs_dir: PathBuf,
}

impl FileSongRepository {
    pub fn new(songs_dir: PathBuf) -> Self {
        Self { songs_dir }
    }

    pub fn songs_dir(&self) -> &Path {
        &self.songs_dir
    }

    pub fn read_all(&self) -> io::Result<Vec<Song>> {
        fs::create_dir_all(&self.songs_dir)?;
        let mut songs = Vec::new();

        for entry in fs::read_dir(&self.songs_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("chordpro") {
                continue;
            }
            songs.push(self.read_song(path, parse_song)?);
        }

        songs.sort_by(|left, right| right.last_modified.cmp(&left.last_modified));
        Ok(songs)
    }

    pub fn read_song<F>(&self, path: impl AsRef<Path>, parser: F) -> io::Result<Song>
    where
        F: Fn(&str, String, u64, u64) -> Song,
    {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        let metadata = fs::metadata(path)?;
        let created = metadata.created().unwrap_or_else(|_| metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH));
        let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);

        Ok(parser(
            &path.to_string_lossy(),
            content,
            created.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
            modified.duration_since(SystemTime::UNIX_EPOCH).unwrap_or_default().as_secs(),
        ))
    }

    pub fn write_song(&self, path: &str, content: &str) -> io::Result<()> {
        fs::write(path, content)
    }
}
