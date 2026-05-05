# Build
cargo build --release

# Run with sample file
cargo run -- -i examples/sample.vtt

# Run with your actual file and merge lines
cargo run -- -i path/to/your/file.vtt -m -o output.txt

# Run tests
cargo test


## Usage_part2

# Basic with timestamps
cargo run -- -i examples/sample.vtt -t

# Merge lines from same speaker
cargo run -- -i examples/sample.vtt -m

# With llama.cpp cleanup (requires llama-server running)
cargo run -- -i examples/sample.vtt -l http://localhost:8080

# Combined: merge, timestamps, markdown output
cargo run -- -i examples/sample.vtt -m -t -f markdown -o output.md

part_3
cargo run -- -i examples/00sample.vtt -d -m
```

This will:
1. **Deduplicate** (`-d`) - Remove overlapping/duplicate captions
2. **Merge** (`-m`) - Merge consecutive lines from the same speaker
3. **Colored output** - Automatically enabled for terminal output (speakers in cyan/bold, timestamps dimmed)

You should now see clean, formatted output with:
-  No duplicates
-  Speaker names in **cyan bold**
-  Timestamps in gray (if you add `-t`)
-  Progress indicators when using llama.cpp

Try these commands:

```bash
# Basic with deduplication and merge
cargo run -- -i examples/00sample.vtt -d -m

# With timestamps
cargo run -- -i examples/00sample.vtt -d -m -t

# With llama.cpp cleanup
cargo run -- -i examples/00sample.vtt -d -m -l http://localhost:8080

# Save to file (no colors)
cargo run -- -i examples/00sample.vtt -d -m -o output.txt

# Force no colors even for terminal
cargo run -- -i examples/00sample.vtt -d -m --no-color
