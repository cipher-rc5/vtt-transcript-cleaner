/// file: src/lib.rs
/// description: Library root exposing public API for VTT transcript cleaning
/// reference: https://github.com/cipher-tech/vtt-transcript-cleaner

pub mod cleaner;
pub mod llama;
pub mod output;
pub mod parser;

pub use cleaner::clean_transcript;
pub use llama::llama_cleanup;
pub use output::TranscriptOutput;
pub use parser::{parse_vtt, VttEntry};
