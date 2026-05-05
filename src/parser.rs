use anyhow::Result;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct VttEntry {
    pub timestamp: String,
    pub speaker: Option<String>,
    pub text: String,
}

pub fn parse_vtt(content: &str) -> Result<Vec<VttEntry>> {
    let mut entries = Vec::new();

    let timestamp_re = Regex::new(
        r"(\d{2}:\d{2}:\d{2}\.\d{3})\s*-->\s*(\d{2}:\d{2}:\d{2}\.\d{3})"
    )?;
    // Matches ">> Name: " or "> Name: " with a named speaker.
    let named_speaker_re = Regex::new(r"^>>?\s*(.+?):\s+")?;
    let tag_re = Regex::new(r"<[^>]+>")?;

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;
    // The last rendered line of the previous cue — used to strip carry-over lines.
    let mut prev_last_line = String::new();

    while i < lines.len() {
        let line = lines[i].trim();

        if line.is_empty()
            || line.starts_with("WEBVTT")
            || line.starts_with("Kind:")
            || line.starts_with("Language:")
        {
            i += 1;
            continue;
        }

        if let Some(caps) = timestamp_re.captures(line) {
            let start_ms = parse_timestamp_ms(caps.get(1).unwrap().as_str()).unwrap_or(0);
            let end_ms = parse_timestamp_ms(caps.get(2).unwrap().as_str()).unwrap_or(0);
            let timestamp = line.to_string();
            i += 1;

            // Collect raw text lines until blank line or next timestamp.
            let mut raw_lines: Vec<&str> = Vec::new();
            while i < lines.len() {
                let tl = lines[i].trim();
                if tl.is_empty() || timestamp_re.is_match(tl) {
                    break;
                }
                raw_lines.push(tl);
                i += 1;
            }

            // YouTube uses 10ms "snapshot" cues that echo the previous cue — skip them.
            if end_ms.saturating_sub(start_ms) < 100 {
                continue;
            }

            if raw_lines.is_empty() {
                continue;
            }

            // Strip all inline tags and decode HTML entities from each line.
            let clean: Vec<String> = raw_lines
                .iter()
                .map(|l| decode_html_entities(tag_re.replace_all(l, "").trim()))
                .filter(|s| !s.is_empty())
                .collect();

            if clean.is_empty() {
                continue;
            }

            // The first line of a multi-line cue is the previous cue's text carried over.
            // Strip it so we only process genuinely new content.
            // Also capture the carry-over's speaker context: YouTube only marks `>>`
            // at the start of a sentence, so a continuation line on the next cue may
            // lack `>>` even though it belongs to the same `>>` speaker.
            let (start_idx, inherited_speaker) = if clean.len() > 1 && clean[0] == prev_last_line {
                let spk = if clean[0].starts_with(">>") { Some(String::new()) } else { None };
                (1, spk)
            } else {
                (0, None)
            };
            let new_lines = &clean[start_idx..];

            // The last clean line of this cue becomes the carry-over for the next cue.
            prev_last_line = clean.last().cloned().unwrap_or_default();

            if new_lines.is_empty() {
                continue;
            }

            // Classify each new line by speaker, grouping consecutive same-speaker lines.
            // A single cue may contain lines from two different speakers (e.g., one `>>`
            // and one plain line), so we emit a separate entry for each speaker segment.
            let mut cur_spk: Option<String> = None;
            let mut cur_txt = String::new();

            let flush = |entries: &mut Vec<VttEntry>, ts: &str, spk: Option<String>, txt: String| {
                let t = txt.trim().to_string();
                if !t.is_empty() {
                    entries.push(VttEntry { timestamp: ts.to_string(), speaker: spk, text: t });
                }
            };

            for line in new_lines {
                let line = line.trim();
                if line.is_empty() { continue; }
                let (spk, txt) = classify_line(&named_speaker_re, line, &inherited_speaker);

                if cur_txt.is_empty() {
                    cur_spk = spk;
                    cur_txt = txt;
                } else if spk == cur_spk {
                    cur_txt.push(' ');
                    cur_txt.push_str(&txt);
                } else {
                    // Speaker changed within this cue — emit current segment, start new.
                    let prev_spk = cur_spk.take();
                    let prev_txt = std::mem::take(&mut cur_txt);
                    flush(&mut entries, &timestamp, prev_spk, prev_txt);
                    cur_spk = spk;
                    cur_txt = txt;
                }
            }
            flush(&mut entries, &timestamp, cur_spk, cur_txt);
        } else {
            i += 1;
        }
    }

    Ok(entries)
}

/// Classify a single clean line into (speaker, text):
/// - `">> Name: text"` → `(Some("Name"), "text")`
/// - `">> text"` (unnamed speaker change) → `(Some(""), "text")`
/// - `"text"` with no `>>` → inherits `inherited_speaker` from the carry-over line.
///   YouTube only puts `>>` at the start of a sentence; a continuation line on the
///   next cue omits `>>` even though it's the same secondary speaker.
fn classify_line(named_re: &Regex, line: &str, inherited_speaker: &Option<String>) -> (Option<String>, String) {
    if let Some(m) = named_re.find(line) {
        let name = named_re
            .captures(line)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().trim().to_string());
        let rest = line[m.end()..].trim().to_string();
        (name, rest)
    } else if line.starts_with(">>") {
        (Some(String::new()), line[2..].trim().to_string())
    } else {
        // No explicit speaker marker — inherit speaker from carry-over context.
        (inherited_speaker.clone(), line.to_string())
    }
}

fn parse_timestamp_ms(ts: &str) -> Option<u64> {
    let mut parts = ts.splitn(3, ':');
    let h: u64 = parts.next()?.parse().ok()?;
    let m: u64 = parts.next()?.parse().ok()?;
    let s_part = parts.next()?;
    let mut s_split = s_part.splitn(2, '.');
    let s: u64 = s_split.next()?.parse().ok()?;
    let ms: u64 = s_split.next()?.parse().ok()?;
    Some(h * 3_600_000 + m * 60_000 + s * 1_000 + ms)
}

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
        let vtt = "WEBVTT\nKind: captions\n\n00:00:07.440 --> 00:00:10.390\nHey everyone, today's DeFi drop\n\n00:00:10.390 --> 00:00:13.350\n>> Alex: Welcome to the show\n";
        let entries = parse_vtt(vtt).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hey everyone, today's DeFi drop");
        assert_eq!(entries[1].speaker, Some("Alex".to_string()));
        assert_eq!(entries[1].text, "Welcome to the show");
    }

    #[test]
    fn test_unnamed_speaker_change() {
        let vtt = "WEBVTT\n\n00:00:00.000 --> 00:00:03.000\nHello.\n\n00:00:03.000 --> 00:00:06.000\n>> Thanks for having me.\n";
        let entries = parse_vtt(vtt).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].speaker, None);
        assert_eq!(entries[1].speaker, Some(String::new()));
        assert_eq!(entries[1].text, "Thanks for having me.");
    }

    #[test]
    fn test_snapshot_cues_skipped() {
        let vtt = "WEBVTT\n\n00:00:07.440 --> 00:00:10.390\nFirst line\n\n00:00:10.390 --> 00:00:10.400\nFirst line\n\n00:00:10.400 --> 00:00:13.350\nFirst line\nSecond line\n";
        let entries = parse_vtt(vtt).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "First line");
        assert_eq!(entries[1].text, "Second line");
    }

    #[test]
    fn test_carryover_stripped() {
        let vtt = "WEBVTT\n\n00:00:00.000 --> 00:00:03.000\nHello world\n\n00:00:03.000 --> 00:00:06.000\nHello world\nThis is new content\n";
        let entries = parse_vtt(vtt).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Hello world");
        assert_eq!(entries[1].text, "This is new content");
    }

    #[test]
    fn test_multi_speaker_within_cue() {
        // A single cue containing both a carry-over and two speaker lines.
        let vtt = "WEBVTT\n\n00:00:00.000 --> 00:00:03.000\nLast sentence.\n\n00:00:03.000 --> 00:00:07.000\nLast sentence.\n>> Response one.\n>> Response two.\n";
        let entries = parse_vtt(vtt).unwrap();
        // First cue: 1 entry
        // Second cue: carry-over stripped, both ">>" lines are same speaker → merged into 1
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].text, "Last sentence.");
        assert_eq!(entries[1].speaker, Some(String::new()));
        assert_eq!(entries[1].text, "Response one. Response two.");
    }

    #[test]
    fn test_parse_timestamp_ms() {
        assert_eq!(parse_timestamp_ms("00:00:07.440"), Some(7440));
        assert_eq!(parse_timestamp_ms("00:01:00.000"), Some(60_000));
        assert_eq!(parse_timestamp_ms("01:00:00.000"), Some(3_600_000));
    }
}
