/// file: src/main.rs
/// description: Main entry point for VTT transcript cleaner CLI application
/// reference: https://github.com/cipher-tech/vtt-transcript-cleaner

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use vtt_transcript_cleaner::{clean_transcript, normalize_transcript, parse_vtt, llama_cleanup};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input VTT file path
    #[arg(short, long)]
    input: PathBuf,

    /// Output file path (optional, defaults to stdout)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: text, json, markdown
    #[arg(short, long, default_value = "text")]
    format: String,

    /// Remove speaker labels
    #[arg(short = 's', long)]
    remove_speakers: bool,

    /// Merge consecutive lines from same speaker
    #[arg(short = 'm', long)]
    merge_lines: bool,

    /// Include timestamps in output
    #[arg(short = 't', long)]
    include_timestamps: bool,

    /// Use llama.cpp for text cleanup (provide llama.cpp server URL)
    #[arg(short = 'l', long)]
    llama_url: Option<String>,

    /// System prompt for llama.cpp cleanup
    #[arg(long, default_value = "You are post-processing a transcript that has already been structurally cleaned. The text is split into paragraphs — each paragraph is one speaker turn. Your tasks: (1) Remove or rewrite bracketed sound effects like [snorts], [laughter], [clears throat] — drop them unless they add meaning. (2) If a speaker's name is clearly established by context, replace unnamed paragraph breaks with 'Name: ' attribution. (3) Fix sentence flow where a sentence is split across a paragraph boundary. (4) Lightly correct grammar and word choice where the meaning is clear. Do NOT add, invent, or summarize content. Preserve all technical terms, proper nouns, and numbers exactly.")]
    llama_prompt: String,

    /// Normalize text: strip filler words (um/uh), remove consecutive duplicate
    /// words caused by rolling-caption carry-over, and capitalize entries
    #[arg(short = 'n', long)]
    normalize: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,

    /// Remove duplicate/overlapping text
    #[arg(short = 'd', long)]
    deduplicate: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Force colored output if not explicitly disabled
    if !args.no_color {
        colored::control::set_override(true);
    }

    // Read input file
    let content = fs::read_to_string(&args.input)?;

    // Parse VTT
    let entries = parse_vtt(&content)?;

    // Clean transcript
    let mut transcript = clean_transcript(
        entries,
        args.remove_speakers,
        args.merge_lines,
        args.deduplicate,
    );

    // Apply rule-based normalization if requested
    if args.normalize {
        transcript = normalize_transcript(transcript);
    }

    // Apply llama.cpp cleanup if requested
    if let Some(llama_url) = &args.llama_url {
        transcript = llama_cleanup(&transcript, llama_url, &args.llama_prompt).await?;
    }

    // Format output
    let output = match args.format.as_str() {
        "json" => transcript.to_json(args.include_timestamps)?,
        "markdown" => transcript.to_markdown(args.include_timestamps),
        _ => {
            if args.no_color || args.output.is_some() {
                transcript.to_text(args.include_timestamps)
            } else {
                transcript.to_text_colored(args.include_timestamps)
            }
        }
    };

    // Write output
    match args.output {
        Some(path) => {
            fs::write(path, output)?;
        }
        None => {
            println!("{}", output);
        }
    }

    Ok(())
}
