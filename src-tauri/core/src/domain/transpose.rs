const SHARPS: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];
const FLATS: [&str; 12] = ["C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B"];

fn note_index(note: &str) -> Option<i32> {
    match note {
        "C" => Some(0),
        "C#" | "Db" => Some(1),
        "D" => Some(2),
        "D#" | "Eb" => Some(3),
        "E" => Some(4),
        "F" => Some(5),
        "F#" | "Gb" => Some(6),
        "G" => Some(7),
        "G#" | "Ab" => Some(8),
        "A" => Some(9),
        "A#" | "Bb" => Some(10),
        "B" => Some(11),
        _ => None,
    }
}

fn transpose_note(note: &str, semitones: i32, prefer_flats: bool) -> Option<String> {
    let next = (note_index(note)? + semitones).rem_euclid(12) as usize;
    Some(if prefer_flats { FLATS[next] } else { SHARPS[next] }.to_string())
}

fn split_root(chord: &str) -> Option<(&str, &str)> {
    let bytes = chord.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    let first = bytes[0] as char;
    if !(('A'..='G').contains(&first)) {
        return None;
    }

    let root_len = if bytes.get(1).is_some_and(|byte| *byte == b'#' || *byte == b'b') { 2 } else { 1 };
    Some(chord.split_at(root_len))
}

pub fn transpose_chord_symbol(chord: &str, semitones: i32) -> String {
    if let Some((head, tail)) = chord.split_once('/') {
        return format!(
            "{}/{}",
            transpose_chord_symbol(head, semitones),
            transpose_chord_symbol(tail, semitones)
        );
    }

    let Some((root, suffix)) = split_root(chord) else {
        return chord.to_string();
    };

    let prefer_flats = root.contains('b');
    let transposed_root = transpose_note(root, semitones, prefer_flats).unwrap_or_else(|| root.to_string());
    format!("{transposed_root}{suffix}")
}

pub fn transpose_bracketed_line(line: &str, semitones: i32) -> String {
    let mut output = String::with_capacity(line.len());
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '[' {
            let mut chord = String::new();
            while let Some(&next) = chars.peek() {
                chars.next();
                if next == ']' {
                    break;
                }
                chord.push(next);
            }
            output.push('[');
            output.push_str(&transpose_chord_symbol(&chord, semitones));
            output.push(']');
        } else {
            output.push(ch);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::{transpose_bracketed_line, transpose_chord_symbol};

    #[test]
    fn transposes_extended_chords_and_slash_chords() {
        assert_eq!(transpose_chord_symbol("Em7", 1), "Fm7");
        assert_eq!(transpose_chord_symbol("Cadd9/G", 2), "Dadd9/A");
        assert_eq!(transpose_chord_symbol("Bbmaj7", -2), "Abmaj7");
        assert_eq!(transpose_chord_symbol("F#dim", 1), "Gdim");
    }

    #[test]
    fn transposes_chords_inside_chordpro_lines() {
        assert_eq!(
            transpose_bracketed_line("[Em7]Today [G/B]is gonna be the day", 2),
            "[F#m7]Today [A/C#]is gonna be the day"
        );
    }
}
