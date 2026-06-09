use serde::Serialize;

#[derive(Debug, Clone)]
pub struct Song {
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
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderedLine {
    pub kind: RenderedLineKind,
    pub label: Option<String>,
    pub chord_line: Option<String>,
    pub lyric_line: Option<String>,
    pub chorus: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RenderedLineKind {
    Section,
    Meta,
    Lyric,
}
