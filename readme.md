# VTT Transcript Cleaner

Rust CLI tool for cleaning and formatting WebVTT (VTT) transcript files. Removes timestamps, HTML tags, and formats dialogue into readable text.

## Features

- Parse WebVTT subtitle/caption files
- Extract clean dialogue without timestamps
- Preserve or remove speaker labels
- Merge consecutive lines from the same speaker
- Multiple output formats: plain text, JSON, Markdown
- Fast and memory-efficient

## Installation
```bash
cargo build --release
```

## Usage

### Basic Usage
```bash
# Output to stdout
cargo run -- -i input.vtt

# Save to file
cargo run -- -i input.vtt -o output.txt

# Merge consecutive lines from same speaker
cargo run -- -i input.vtt -m -o output.txt

# Remove speaker labels
cargo run -- -i input.vtt -s -o output.txt
```

### Output Formats
```bash
# Plain text (default)
cargo run -- -i input.vtt -f text -o output.txt

# JSON
cargo run -- -i input.vtt -f json -o output.json

# Markdown
cargo run -- -i input.vtt -f markdown -o output.md
```

## Example

**Input (VTT):**
```
WEBVTT

00:00:07.440 --> 00:00:10.390
>> Alex: Welcome to the show

00:00:10.400 --> 00:00:13.350
Today we're discussing DeFi
```

**Output (Text):**
```
Alex: Welcome to the show

Today we're discussing DeFi
```

## Library Usage
```rust
use vtt_transcript_cleaner::{parse_vtt, clean_transcript};

let vtt_content = std::fs::read_to_string("input.vtt")?;
let entries = parse_vtt(&vtt_content)?;
let transcript = clean_transcript(entries, false, true);
let text = transcript.to_text();
println!("{}", text);
```

## License

MIT

## Author

Cipher - https://github.com/cipher-rc5
