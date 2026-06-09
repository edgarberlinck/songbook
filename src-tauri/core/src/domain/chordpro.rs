use crate::domain::song::{RenderedLine, RenderedLineKind, Song};

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut previous_dash = false;

    for ch in value.chars().flat_map(|ch| ch.to_lowercase()) {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch);
            previous_dash = false;
        } else if !previous_dash {
            slug.push('-');
            previous_dash = true;
        }
    }

    slug.trim_matches('-').to_string()
}

fn parse_directive(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
        return None;
    }

    let inner = &trimmed[1..trimmed.len() - 1];
    let (key, value) = inner.split_once(':')?;
    Some((key.trim(), value.trim()))
}

fn render_chord_line(line: &str) -> (String, String) {
    let mut lyric = String::new();
    let mut chords: Vec<char> = Vec::new();
    let mut current_chord: Option<String> = None;

    for ch in line.chars() {
        match ch {
            '[' => current_chord = Some(String::new()),
            ']' => {
                if let Some(chord) = current_chord.take() {
                    let start = lyric.chars().count();
                    while chords.len() < start {
                        chords.push(' ');
                    }
                    for (offset, chord_char) in chord.chars().enumerate() {
                        if chords.len() <= start + offset {
                            chords.push(chord_char);
                        } else {
                            chords[start + offset] = chord_char;
                        }
                    }
                }
            }
            _ => {
                if let Some(buffer) = current_chord.as_mut() {
                    buffer.push(ch);
                } else {
                    lyric.push(ch);
                    if chords.len() < lyric.chars().count() {
                        chords.push(' ');
                    }
                }
            }
        }
    }

    (chords.into_iter().collect::<String>().trim_end().to_string(), lyric)
}

pub fn parse_song(path: &str, content: String, created_at: u64, last_modified: u64) -> Song {
    let fallback_title = std::path::Path::new(path)
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("Untitled")
        .replace('-', " ");

    let mut title = fallback_title.clone();
    let mut subtitle = None;
    let mut artist = None;
    let mut album = None;
    let mut key = None;
    let mut capo = None;
    let mut tempo = None;
    let mut tags: Vec<String> = Vec::new();
    let mut notes = None;
    let mut favorite = false;
    let mut preview = Vec::new();
    let mut in_chorus = false;

    for raw_line in content.lines() {
        if let Some((directive, value)) = parse_directive(raw_line) {
            match directive.to_ascii_lowercase().as_str() {
                "title" | "t" => title = value.to_string(),
                "subtitle" | "st" => subtitle = Some(value.to_string()),
                "artist" => artist = Some(value.to_string()),
                "album" => album = Some(value.to_string()),
                "key" => key = Some(value.to_string()),
                "capo" => capo = value.parse::<i32>().ok(),
                "tempo" => tempo = value.parse::<i32>().ok(),
                "tags" => {
                    tags = value
                        .split(',')
                        .map(|item| item.trim())
                        .filter(|item| !item.is_empty())
                        .map(ToOwned::to_owned)
                        .collect()
                }
                "notes" => notes = Some(value.to_string()),
                "favorite" => favorite = matches!(value, "true" | "yes" | "1"),
                "comment" | "c" => preview.push(RenderedLine {
                    kind: RenderedLineKind::Section,
                    label: Some(value.to_string()),
                    chord_line: None,
                    lyric_line: None,
                    chorus: in_chorus,
                }),
                "start_of_chorus" | "soc" => in_chorus = true,
                "end_of_chorus" | "eoc" => in_chorus = false,
                _ => preview.push(RenderedLine {
                    kind: RenderedLineKind::Meta,
                    label: Some(format!("{}: {}", directive, value)),
                    chord_line: None,
                    lyric_line: None,
                    chorus: in_chorus,
                }),
            }
            continue;
        }

        if raw_line.trim().is_empty() {
            preview.push(RenderedLine {
                kind: RenderedLineKind::Lyric,
                label: None,
                chord_line: None,
                lyric_line: Some(String::new()),
                chorus: in_chorus,
            });
            continue;
        }

        let (chord_line, lyric_line) = render_chord_line(raw_line);
        preview.push(RenderedLine {
            kind: RenderedLineKind::Lyric,
            label: None,
            chord_line: if chord_line.trim().is_empty() { None } else { Some(chord_line) },
            lyric_line: Some(lyric_line),
            chorus: in_chorus,
        });
    }

    let id = slugify(&title);

    Song {
        id,
        title,
        subtitle,
        artist,
        album,
        key,
        capo,
        tempo,
        tags,
        notes,
        favorite,
        created_at,
        last_modified,
        content,
        preview,
        path: path.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_song;

    #[test]
    fn parses_metadata_and_preview_from_chordpro() {
        let song = parse_song(
            "songs/wonderwall.chordpro",
            "{title: Wonderwall}\n{artist: Oasis}\n{key: Em}\n{capo: 2}\n{tags: Britpop, Acoustic}\n{comment: Verse 1}\n[Em7]Today is gonna be the day\n".to_string(),
            1,
            2,
        );

        assert_eq!(song.id, "wonderwall");
        assert_eq!(song.title, "Wonderwall");
        assert_eq!(song.artist.as_deref(), Some("Oasis"));
        assert_eq!(song.key.as_deref(), Some("Em"));
        assert_eq!(song.capo, Some(2));
        assert_eq!(song.tags, vec!["Britpop", "Acoustic"]);
        assert_eq!(song.preview.len(), 2);
        assert_eq!(song.preview[0].label.as_deref(), Some("Verse 1"));
        assert_eq!(song.preview[1].chord_line.as_deref(), Some("Em7"));
        assert_eq!(song.preview[1].lyric_line.as_deref(), Some("Today is gonna be the day"));
    }
}
