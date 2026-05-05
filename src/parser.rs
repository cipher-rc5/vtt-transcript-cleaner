/// file: src/parser.rs
/// description: VTT file parser that extracts dialogue and speaker information
/// reference: https://www.w3.org/TR/webvtt1/

use anyhow::{ Result};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct VttEntry {
    pub timestamp: String,
    pub speaker: Option<String>,
    pub text: String,
}

pub fn parse_vtt(content: &str) -> Result<Vec<VttEntry>> {
    let mut entries = Vec::new();

    // Regex patterns
    let timestamp_pattern = Regex::new(
        r"(\d{2}:\d{2}:\d{2}\.\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}\.\d{3})"
    )?;

    let speaker_pattern = Regex::new(r"^>>?\s*(.+?)(?::|$)")?;
    let tag_pattern = Regex::new(r"<[^>]+>")?;
    let timestamp_tag_pattern = Regex::new(r"<\d{2}:\d{2}:\d{2}\.\d{3}><c>")?;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let line = lines[i].trim();

        // Skip WEBVTT header and empty lines
        if line.is_empty() || line.starts_with("WEBVTT") || line.starts_with("Kind:") {
            i += 1;
            continue;
        }

        // Check if this is a timestamp line
        if timestamp_pattern.is_match(line) {
            let timestamp = line.to_string();
            i += 1;

            // Collect all text lines until next timestamp or empty line
            let mut text_lines = Vec::new();
            while i < lines.len() {
                let text_line = lines[i].trim();
                if text_line.is_empty() || timestamp_pattern.is_match(text_line) {
                    break;
                }
                text_lines.push(text_line);
                i += 1;
            }

            // Process collected text
            if !text_lines.is_empty() {
                let full_text = text_lines.join(" ");

                // Remove timestamp tags like <00:00:07.440><c>
                let cleaned_text = timestamp_tag_pattern.replace_all(&full_text, "");

                // Remove HTML tags
                let cleaned_text = tag_pattern.replace_all(&cleaned_text, "");

                // Decode HTML entities
                let cleaned_text = decode_html_entities(&cleaned_text);

                // Extract speaker if present
                let (speaker, text) = if let Some(caps) = speaker_pattern.captures(&cleaned_text) {
                    let speaker_name = caps.get(1)
                        .map(|m| m.as_str().trim().to_string());
                    let remaining_text = speaker_pattern.replace(&cleaned_text, "").trim().to_string();
                    (speaker_name, remaining_text)
                } else {
                    (None, cleaned_text.trim().to_string())
                };

                if !text.is_empty() {
                    entries.push(VttEntry {
                        timestamp,
                        speaker,
                        text,
                    });
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(entries)
}

/// Decode common HTML entities
fn decode_html_entities(text: &str) -> String {
    text.replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
        .replace("&#39;", "'")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_vtt() {
        let vtt = r#"WEBVTT
Kind: captions

00:00:07.440 --> 00:00:10.390
Hey everyone, today's DeFi drop

00:00:10.390 --> 00:00:13.350
>> Alex: Welcome to the show
"#;

        let entries = parse_vtt(vtt).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hey everyone, today's DeFi drop");
        assert_eq!(entries[1].speaker, Some("Alex".to_string()));
    }
}
