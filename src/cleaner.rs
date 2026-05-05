/// file: src/cleaner.rs
/// description: Core logic for cleaning and formatting transcript entries
/// reference: https://github.com/cipher-tech/vtt-transcript-cleaner

use crate::output::TranscriptOutput;
use crate::parser::VttEntry;

pub fn clean_transcript(
    entries: Vec<VttEntry>,
    remove_speakers: bool,
    merge_lines: bool,
    deduplicate: bool,
) -> TranscriptOutput {
    let entries = if deduplicate {
        deduplicate_entries(entries)
    } else {
        entries
    };
    let mut cleaned_entries = Vec::new();

    if merge_lines {
        let mut current_speaker: Option<String> = None;
        let mut current_text = String::new();
        let mut current_timestamp = String::new();

        for entry in entries {
            let speaker = if remove_speakers {
                None
            } else {
                entry.speaker.clone()
            };

            if speaker == current_speaker && !current_text.is_empty() {
                // Merge with previous line
                current_text.push(' ');
                current_text.push_str(&entry.text);
            } else {
                // Save previous entry if exists
                if !current_text.is_empty() {
                    cleaned_entries.push(VttEntry {
                        timestamp: current_timestamp.clone(),
                        speaker: current_speaker.clone(),
                        text: current_text.clone(),
                    });
                }

                // Start new entry
                current_speaker = speaker;
                current_text = entry.text.clone();
                current_timestamp = entry.timestamp.clone();
            }
        }

        // Add last entry
        if !current_text.is_empty() {
            cleaned_entries.push(VttEntry {
                timestamp: current_timestamp,
                speaker: current_speaker,
                text: current_text,
            });
        }
    } else {
        for entry in entries {
            cleaned_entries.push(VttEntry {
                timestamp: entry.timestamp,
                speaker: if remove_speakers {
                    None
                } else {
                    entry.speaker
                },
                text: entry.text,
            });
        }
    }

    TranscriptOutput {
        entries: cleaned_entries,
    }
}

/// Remove duplicate/overlapping text from entries
fn deduplicate_entries(entries: Vec<VttEntry>) -> Vec<VttEntry> {
    if entries.is_empty() {
        return entries;
    }

    let mut deduped = Vec::new();
    let mut last_text = String::new();

    for entry in entries {
        let text = entry.text.trim();

        // Skip if this text is a substring of the last text (overlapping caption)
        if last_text.contains(text) && !text.is_empty() {
            continue;
        }

        // Skip if the last text is a substring of this text (this is an expansion)
        if text.contains(&last_text) && !last_text.is_empty() {
            // Replace the last entry with this expanded version
            deduped.pop();
        }

        // Skip exact duplicates
        if text == last_text {
            continue;
        }

        deduped.push(entry.clone());
        last_text = text.to_string();
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merge_consecutive_lines() {
        let entries = vec![
            VttEntry {
                timestamp: "00:00:01".to_string(),
                speaker: Some("Alex".to_string()),
                text: "Hello".to_string(),
            },
            VttEntry {
                timestamp: "00:00:02".to_string(),
                speaker: Some("Alex".to_string()),
                text: "World".to_string(),
            },
        ];

        let result = clean_transcript(entries, false, true, false);
        assert_eq!(result.entries.len(), 1);
        assert_eq!(result.entries[0].text, "Hello World");
    }
}
