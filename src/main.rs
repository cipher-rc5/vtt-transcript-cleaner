/// file: src/main.rs
/// description: Main entry point for VTT transcript cleaner CLI application
/// reference: https://github.com/cipher-tech/vtt-transcript-cleaner

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;
use vtt_transcript_cleaner::{clean_transcript, parse_vtt, llama_cleanup};

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
    #[arg(long, default_value = "Clean up the following transcript text by fixing grammar, removing duplicates, and making it more readable. Preserve the original meaning and speaker attributions.")]
    llama_prompt: String,

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
