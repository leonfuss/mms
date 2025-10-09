# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

MMS (My Study Management System) - A Rust CLI application for managing university studies on macOS.

**Key Features:**
- Automatic symlink management based on course schedules
- Todo tracking per lecture and course
- Git auto-commits with lecture numbering
- Grade and ECTS tracking with category requirements
- Holiday management with course-specific exceptions
- Background service for automatic course switching

See `TODO.md` for detailed implementation plan and architecture.

## Development Commands

### Building
```bash
cargo build
```

### Running
```bash
cargo run
```

### Testing
```bash
cargo test
```

### Checking (fast compile check without building)
```bash
cargo check
```

### Linting
```bash
cargo clippy
```

### Formatting
```bash
cargo fmt
```

### Running a single test
```bash
cargo test test_name
```

## Code Structure

The project currently has a minimal structure:
- `src/main.rs` - Entry point with basic hello world implementation
- `Cargo.toml` - Project manifest (edition 2024, no dependencies yet)
