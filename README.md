# Songbook

Songbook is a Tauri 2 desktop application for managing a local-first library of ChordPro songs.

## Current foundation

This repository now includes an initial vertical slice with:

- Tauri 2 + React + TypeScript application shell
- Clean Architecture-inspired Rust modules (`domain`, `application`, `infrastructure`, `ui`)
- Local-first `songs/` directory where each song is stored as a human-readable `.chordpro` file
- SQLite metadata index and migrations for songs, tags, setlists, settings, and setlist songs
- Sidebar library, search, favorites filter, tag filter, sort control, and multi-selection UI
- Split editor / live preview with autosave, zoom, chord visibility, and reading mode
- Chord transposition for standard ChordPro chord tokens including slash chords
- Rust domain tests for parsing and transposition

## Development

```bash
npm install
npm run tauri dev
```

## Verification

```bash
npm run lint
npm run build
cargo test --manifest-path src-tauri/Cargo.toml
```
