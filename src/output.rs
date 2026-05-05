/// file: src/output.rs
/// description: Output formatters for cleaned transcripts (text, JSON, markdown)
/// reference: https://github.com/cipher-tech/vtt-transcript-cleaner

use crate::parser::VttEntry;
use anyhow::Result;
use colored::*;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct TranscriptOutput {
    pub entries: Vec<VttEntry>,
}

impl Serialize for VttEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("VttEntry", 2)?;
        state.serialize_field("speaker", &self.speaker)?;
        state.serialize_field("text", &self.text)?;
        state.end()
    }
}

impl TranscriptOutput {
    pub fn to_text(&self, include_timestamps: bool) -> String {
        let mut output = String::new();

        for entry in &self.entries {
            if include_timestamps && !entry.timestamp.is_empty() {
                output.push('[');
                output.push_str(&entry.timestamp);
                output.push_str("] ");
            }

            // Named speaker only — empty string means unnamed speaker change, no label.
            if let Some(speaker) = &entry.speaker {
                if !speaker.is_empty() {
                    output.push_str(speaker);
                    output.push_str(": ");
                }
            }
            output.push_str(&entry.text);
            output.push_str("\n\n");
        }

        output
    }

    pub fn to_text_colored(&self, include_timestamps: bool) -> String {
        let mut output = String::new();

        for entry in &self.entries {
            let mut line = String::new();

            if include_timestamps && !entry.timestamp.is_empty() {
                line.push_str(&format!("{} ", format!("[{}]", entry.timestamp).dimmed()));
            }

            if let Some(speaker) = &entry.speaker {
                if !speaker.is_empty() {
                    line.push_str(&format!("{} ", format!("{}:", speaker).cyan().bold()));
                }
            }
            line.push_str(&entry.text);

            output.push_str(&line);
            output.push_str("\n\n");
        }

        output
    }

    pub fn to_json(&self, include_timestamps: bool) -> Result<String> {
        if include_timestamps {
            Ok(serde_json::to_string_pretty(self)?)
        } else {
            // Create a version without timestamps
            let entries_without_timestamps: Vec<_> = self.entries
                .iter()
                .map(|e| {
                    serde_json::json!({
                        "speaker": e.speaker,
                        "text": e.text
                    })
                })
                .collect();
            Ok(serde_json::to_string_pretty(&serde_json::json!({
                "entries": entries_without_timestamps
            }))?)
        }
    }

    pub fn to_markdown(&self, include_timestamps: bool) -> String {
        let mut output = String::from("# Transcript\n\n");

        for entry in &self.entries {
            if include_timestamps && !entry.timestamp.is_empty() {
                output.push_str("_");
                output.push_str(&entry.timestamp);
                output.push_str("_\n\n");
            }

            if let Some(speaker) = &entry.speaker {
                if !speaker.is_empty() {
                    output.push_str("**");
                    output.push_str(speaker);
                    output.push_str("**: ");
                }
            }
            output.push_str(&entry.text);
            output.push_str("\n\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_text_format() {
        let transcript = TranscriptOutput {
            entries: vec![
                VttEntry {
                    timestamp: String::new(),
                    speaker: Some("Alex".to_string()),
                    text: "Hello world".to_string(),
                },
            ],
        };

        let text = transcript.to_text(false);
        assert!(text.contains("Alex: Hello world"));

        let text_with_timestamp = transcript.to_text(true);
        assert!(text_with_timestamp.contains("Alex: Hello world"));
    }
}
