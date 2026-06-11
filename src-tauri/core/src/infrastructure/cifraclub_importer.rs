use reqwest::blocking::Client;
use scraper::{ElementRef, Html, Selector};

pub struct ImportedSongData {
    pub title: String,
    pub artist: Option<String>,
    pub key: Option<String>,
    pub tuning: Option<String>,
    pub capo: Option<i32>,
    pub body: String,
}

pub struct CifraClubImporter;

impl CifraClubImporter {
    pub fn import_from_url(url: &str) -> Result<ImportedSongData, String> {
        if !url.contains("cifraclub.com") {
            return Err("Only Cifra Club URLs are currently supported".to_string());
        }

        let client = Client::builder()
            .user_agent("curl/8.7.1")
            .build()
            .map_err(|error| error.to_string())?;

        let html = client
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(|error| error.to_string())?
            .text()
            .map_err(|error| error.to_string())?;

        let document = Html::parse_document(&html);
        let title = Self::extract_title(&document)?;
        let artist = Self::extract_text(&document, "h2.t3 a");
        let key = Self::extract_text(&document, "#cifra_tom");
        let tuning = Self::extract_text(&document, "#cifra_afi");
        let capo = Self::extract_text(&document, "#cifra_capo").and_then(|value| Self::extract_first_integer(&value));

        let pre_selector = Selector::parse("#cifra > div.cifra-column--left > div > div > pre")
            .map_err(|error| error.to_string())?;
        let pre = document
            .select(&pre_selector)
            .next()
            .ok_or_else(|| "Could not find the song content in the HTML".to_string())?;

        let body = Self::render_element(pre)
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        if body.is_empty() {
            return Err("Imported song content is empty".to_string());
        }

        Ok(ImportedSongData {
            title,
            artist,
            key,
            tuning,
            capo,
            body,
        })
    }

    fn extract_title(document: &Html) -> Result<String, String> {
        let song_header = Self::extract_text(document, "h1.t1");
        if let Some(title) = song_header {
            if !title.is_empty() {
                return Ok(title);
            }
        }

        let title_tag = Self::extract_text(document, "title")
            .ok_or_else(|| "Could not identify the song title".to_string())?;

        let cleaned = title_tag
            .split(" - Cifra Club")
            .next()
            .unwrap_or(title_tag.as_str())
            .split(" - ")
            .next()
            .unwrap_or(title_tag.as_str())
            .trim()
            .to_string();

        if cleaned.is_empty() {
            Err("Could not identify the song title".to_string())
        } else {
            Ok(cleaned)
        }
    }

    fn extract_text(document: &Html, selector: &str) -> Option<String> {
        let selector = Selector::parse(selector).ok()?;
        document
            .select(&selector)
            .next()
            .map(|element| element.text().collect::<String>().trim().to_string())
            .filter(|value| !value.is_empty())
    }

    fn extract_first_integer(value: &str) -> Option<i32> {
        let mut digits = String::new();
        for ch in value.chars() {
            if ch.is_ascii_digit() {
                digits.push(ch);
            } else if !digits.is_empty() {
                break;
            }
        }

        if digits.is_empty() {
            None
        } else {
            digits.parse::<i32>().ok()
        }
    }

    fn render_element(element: ElementRef<'_>) -> String {
        let mut output = String::new();
        for child in element.children() {
            output.push_str(&Self::render_node(child));
        }
        output
    }

    fn render_node(node: ego_tree::NodeRef<'_, scraper::node::Node>) -> String {
        match node.value() {
            scraper::node::Node::Text(text) => text.to_string(),
            scraper::node::Node::Element(_) => {
                let Some(element) = ElementRef::wrap(node) else {
                    return String::new();
                };

                let name = element.value().name();
                if name.eq_ignore_ascii_case("br") {
                    return "\n".to_string();
                }

                let class_attr = element.value().attr("class").unwrap_or("");
                let is_tablature = class_attr.split_whitespace().any(|class_name| class_name == "tablatura" || class_name == "cnt");

                if name.eq_ignore_ascii_case("b") {
                    let chord = element.text().collect::<String>().trim().to_string();
                    if chord.is_empty() {
                        return String::new();
                    }
                    return format!("[{chord}]");
                }

                let mut content = String::new();
                for child in element.children() {
                    content.push_str(&Self::render_node(child));
                }

                if is_tablature {
                    let trimmed = content.trim();
                    if trimmed.is_empty() {
                        String::new()
                    } else {
                        format!("{trimmed}\n")
                    }
                } else {
                    content
                }
            }
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CifraClubImporter;

    #[test]
    #[ignore = "Runs a live network call against Cifra Club"]
    fn imports_live_cifraclub_song() {
        let imported = CifraClubImporter::import_from_url(
            "https://www.cifraclub.com.br/bruno-e-marrone/boate-azul/",
        )
        .expect("should import the song from the live page");

        assert!(imported.title.to_lowercase().contains("boate azul"));
        assert!(imported.body.contains("["));
        assert!(!imported.body.trim().is_empty());
    }
}