/// file: src/llama.rs
/// description: llama.cpp integration for AI-powered text cleanup
/// reference: https://github.com/ggerganov/llama.cpp

use anyhow::{Context, Result};
use colored::*;
use serde::{Deserialize, Serialize};
use crate::output::TranscriptOutput;
use crate::parser::VttEntry;

#[derive(Debug, Serialize)]
struct LlamaRequest {
    prompt: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_prompt: Option<String>,
    temperature: f32,
    top_p: f32,
    n_predict: i32,
    stop: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct LlamaResponse {
    content: String,
}

/// Clean up transcript text using llama.cpp server
pub async fn llama_cleanup(
    transcript: &TranscriptOutput,
    llama_url: &str,
    system_prompt: &str,
) -> Result<TranscriptOutput> {
    let client = reqwest::Client::new();

    // Convert transcript to text for processing (without timestamps to save tokens)
    let input_text = transcript.to_text(false);

    // Estimate tokens (roughly 4 chars per token)
    let estimated_tokens = input_text.len() / 4;

    // If text is too long, process in chunks
    if estimated_tokens > 6000 {
        eprintln!("{}", format!("⚠  Transcript is large (~{} tokens), processing in chunks...", estimated_tokens).yellow());
        return process_in_chunks(transcript, llama_url, system_prompt, &client).await;
    }

    eprintln!("{}", "✓ Sending transcript to llama.cpp...".green());

    // Prepare the request
    let request = LlamaRequest {
        prompt: format!("{}\n\n{}", system_prompt, input_text),
        system_prompt: None,
        temperature: 0.3,
        top_p: 0.9,
        n_predict: 4096,
        stop: vec!["</s>".to_string()],
    };

    // Send request to llama.cpp server
    let url = format!("{}/completion", llama_url.trim_end_matches('/'));
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to connect to llama.cpp server")?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!(
            "llama.cpp server returned error: {} - {}",
            status,
            error_text
        );
    }

    let llama_response: LlamaResponse = response
        .json()
        .await
        .context("Failed to parse llama.cpp response")?;

    // Parse the cleaned text back into entries
    let cleaned_entries = parse_cleaned_text(&llama_response.content, &transcript.entries);

    Ok(TranscriptOutput {
        entries: cleaned_entries,
    })
}

/// Process large transcripts in chunks
async fn process_in_chunks(
    transcript: &TranscriptOutput,
    llama_url: &str,
    system_prompt: &str,
    client: &reqwest::Client,
) -> Result<TranscriptOutput> {
    let chunk_size = 20; // Process 20 entries at a time
    let mut all_cleaned_entries = Vec::new();
    let total_chunks = (transcript.entries.len() + chunk_size - 1) / chunk_size;

    for (i, chunk) in transcript.entries.chunks(chunk_size).enumerate() {
        eprintln!("{}", format!("  Processing chunk {}/{}...", i + 1, total_chunks).cyan());
        let chunk_transcript = TranscriptOutput {
            entries: chunk.to_vec(),
        };

        let input_text = chunk_transcript.to_text(false);

        let request = LlamaRequest {
            prompt: format!("{}\n\n{}", system_prompt, input_text),
            system_prompt: None,
            temperature: 0.3,
            top_p: 0.9,
            n_predict: 2048,
            stop: vec!["</s>".to_string()],
        };

        let url = format!("{}/completion", llama_url.trim_end_matches('/'));
        let response = client
            .post(&url)
            .json(&request)
            .send()
            .await
            .context("Failed to connect to llama.cpp server")?;

        if !response.status().is_success() {
            eprintln!("{}", format!("  ⚠  Chunk {} failed, using original text", i + 1).yellow());
            all_cleaned_entries.extend(chunk.to_vec());
            continue;
        }

        eprintln!("{}", format!("  ✓ Chunk {} completed", i + 1).green());

        let llama_response: LlamaResponse = response
            .json()
            .await
            .context("Failed to parse llama.cpp response")?;

        let cleaned_entries = parse_cleaned_text(&llama_response.content, chunk);
        all_cleaned_entries.extend(cleaned_entries);
    }

    Ok(TranscriptOutput {
        entries: all_cleaned_entries,
    })
}

/// Parse cleaned text back into structured entries
fn parse_cleaned_text(cleaned_text: &str, original_entries: &[VttEntry]) -> Vec<VttEntry> {
    let mut entries = Vec::new();

    // Split by double newlines or speaker patterns
    let lines: Vec<&str> = cleaned_text
        .split('\n')
        .filter(|l| !l.trim().is_empty())
        .collect();

    for (i, line) in lines.iter().enumerate() {
        let line = line.trim();

        // Try to extract speaker and text
        if let Some((speaker, text)) = parse_speaker_line(line) {
            entries.push(VttEntry {
                timestamp: original_entries
                    .get(i)
                    .map(|e| e.timestamp.clone())
                    .unwrap_or_default(),
                speaker: Some(speaker),
                text: text.to_string(),
            });
        } else {
            entries.push(VttEntry {
                timestamp: original_entries
                    .get(i)
                    .map(|e| e.timestamp.clone())
                    .unwrap_or_default(),
                speaker: None,
                text: line.to_string(),
            });
        }
    }

    entries
}

/// Parse a line like "Speaker: text" into (speaker, text)
fn parse_speaker_line(line: &str) -> Option<(String, String)> {
    if let Some(pos) = line.find(':') {
        let speaker = line[..pos].trim();
        let text = line[pos + 1..].trim();

        // Only treat as speaker if it looks reasonable
        if !speaker.is_empty() && !text.is_empty() && speaker.len() < 50 {
            return Some((speaker.to_string(), text.to_string()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_speaker_line() {
        let result = parse_speaker_line("Alex: Hello world");
        assert_eq!(result, Some(("Alex".to_string(), "Hello world".to_string())));

        let result = parse_speaker_line("Just text without speaker");
        assert_eq!(result, None);
    }
}
