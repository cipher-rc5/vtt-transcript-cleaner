use crate::output::TranscriptOutput;
use crate::parser::VttEntry;

static FILLERS: &[&str] = &["um", "uh", "er", "erm"];

pub fn normalize_transcript(transcript: TranscriptOutput) -> TranscriptOutput {
    TranscriptOutput {
        entries: transcript
            .entries
            .into_iter()
            .map(|e| VttEntry {
                text: normalize_text(&e.text),
                ..e
            })
            .collect(),
    }
}

pub fn normalize_text(text: &str) -> String {
    let t = remove_consecutive_duplicate_words(text);
    let t = remove_filler_words(&t);
    capitalize_first(&t)
}

/// Removes adjacent identical words that are artifacts of the rolling-caption
/// word-reveal format. Does not collapse across sentence boundaries (`.`, `!`, `?`).
fn remove_consecutive_duplicate_words(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut out: Vec<&str> = Vec::with_capacity(words.len());

    for word in &words {
        if let Some(&prev) = out.last() {
            if !prev.ends_with(['.', '!', '?']) {
                let cur = word.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                let pre = prev.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
                if !cur.is_empty() && cur == pre {
                    continue;
                }
            }
        }
        out.push(word);
    }

    out.join(" ")
}

/// Strips standalone filler words (um, uh, er, erm).
fn remove_filler_words(text: &str) -> String {
    text.split_whitespace()
        .filter(|w| {
            let bare = w.trim_matches(|c: char| !c.is_alphanumeric()).to_lowercase();
            !FILLERS.contains(&bare.as_str())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn capitalize_first(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_duplicate_word_removal() {
        assert_eq!(normalize_text("I I know you're not"), "I know you're not");
        assert_eq!(normalize_text("about about 10"), "About 10");
    }

    #[test]
    fn test_no_dedup_across_sentence_boundary() {
        // "10. 10 times" — the second "10" starts a new sentence, keep both.
        assert_eq!(normalize_text("about 10. 10 times"), "About 10. 10 times");
    }

    #[test]
    fn test_filler_removal() {
        assert_eq!(normalize_text("um I think uh we should"), "I think we should");
    }

    #[test]
    fn test_capitalize() {
        assert_eq!(normalize_text("hello world"), "Hello world");
    }
}
