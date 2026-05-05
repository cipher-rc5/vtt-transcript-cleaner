#!/bin/bash

# Create project structure
cargo new vtt-transcript-cleaner
cd vtt-transcript-cleaner

# Create directories
mkdir -p examples tests

# The files above should be created in their respective locations
# Then build and test:
cargo build
cargo test

echo "Project setup complete!"
