use std::path::PathBuf;

use rusqlite::{params, Connection};

use crate::domain::song::Song;

pub struct SqliteIndex {
    db_path: PathBuf,
}

impl SqliteIndex {
    pub fn new(db_path: PathBuf) -> rusqlite::Result<Self> {
        let index = Self { db_path };
        index.migrate()?;
        Ok(index)
    }

    pub fn reset(&self) -> rusqlite::Result<()> {
        let connection = self.connection()?;
        connection.execute("DELETE FROM song_tags", [])?;
        connection.execute("DELETE FROM tags", [])?;
        connection.execute("DELETE FROM songs", [])?;
        Ok(())
    }

    pub fn upsert_song(&self, song: &Song) -> rusqlite::Result<()> {
        let connection = self.connection()?;
        connection.execute(
            "INSERT INTO songs (id, title, subtitle, artist, album, song_key, capo, tempo, notes, favorite, created_at, last_modified, path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
             ON CONFLICT(id) DO UPDATE SET
               title = excluded.title,
               subtitle = excluded.subtitle,
               artist = excluded.artist,
               album = excluded.album,
               song_key = excluded.song_key,
               capo = excluded.capo,
               tempo = excluded.tempo,
               notes = excluded.notes,
               favorite = excluded.favorite,
               created_at = excluded.created_at,
               last_modified = excluded.last_modified,
               path = excluded.path",
            params![
                song.id,
                song.title,
                song.subtitle,
                song.artist,
                song.album,
                song.key,
                song.capo,
                song.tempo,
                song.notes,
                song.favorite,
                song.created_at as i64,
                song.last_modified as i64,
                song.path,
            ],
        )?;

        connection.execute("DELETE FROM song_tags WHERE song_id = ?1", params![song.id])?;
        for tag in &song.tags {
            connection.execute(
                "INSERT OR IGNORE INTO tags (name) VALUES (?1)",
                params![tag],
            )?;
            connection.execute(
                "INSERT INTO song_tags (song_id, tag_name) VALUES (?1, ?2)",
                params![song.id, tag],
            )?;
        }

        Ok(())
    }

    fn connection(&self) -> rusqlite::Result<Connection> {
        Connection::open(&self.db_path)
    }

    fn migrate(&self) -> rusqlite::Result<()> {
        let connection = self.connection()?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS songs (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                subtitle TEXT,
                artist TEXT,
                album TEXT,
                song_key TEXT,
                capo INTEGER,
                tempo INTEGER,
                notes TEXT,
                favorite INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                last_modified INTEGER NOT NULL,
                path TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS tags (
                name TEXT PRIMARY KEY
            );
            CREATE TABLE IF NOT EXISTS song_tags (
                song_id TEXT NOT NULL,
                tag_name TEXT NOT NULL,
                PRIMARY KEY(song_id, tag_name),
                FOREIGN KEY(song_id) REFERENCES songs(id) ON DELETE CASCADE,
                FOREIGN KEY(tag_name) REFERENCES tags(name) ON DELETE CASCADE
            );
            CREATE TABLE IF NOT EXISTS setlists (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                notes TEXT
            );
            CREATE TABLE IF NOT EXISTS setlist_songs (
                setlist_id TEXT NOT NULL,
                song_id TEXT NOT NULL,
                position INTEGER NOT NULL,
                notes TEXT,
                transpose INTEGER NOT NULL DEFAULT 0,
                PRIMARY KEY(setlist_id, song_id, position)
            );
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );",
        )?;
        Ok(())
    }
}
