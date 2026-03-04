# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
cargo build           # debug build
cargo build --release # optimized build
cargo check           # verify compilation without building
cargo clippy          # lint
cargo fmt             # format
cargo test            # run tests (none currently)
```

Run the tool:
```bash
./target/debug/macos-file-rename <FOLDER>
```

## Architecture

Single-file CLI (`src/main.rs`) that renames JPEG/JPG files by prefixing them with their EXIF `DateTimeOriginal` date in `YYYYMMDD-` format (e.g. `photo.jpg` → `20241231-photo.jpg`).

**Flow:** parse args → walk directory recursively → filter `.jpg`/`.jpeg` (case-insensitive) → for each file: check if already prefixed, extract EXIF date, check target doesn't exist, rename.

**Key types:**
- `Args` (clap derive) — single positional `folder: PathBuf`
- `Outcome` — `Renamed` or `Skipped(SkipReason)`
- `SkipReason` — `AlreadyPrefixed`, `NoExifDate`, `TargetExists`, `IoError`

**Key functions:**
- `extract_date(path)` — reads EXIF via `kamadak-exif`, returns `Option<String>` formatted as `YYYYMMDD`
- `already_prefixed(filename)` — checks if filename starts with 8 ASCII digits + `-`
- `process_file(path)` — orchestrates the check/extract/rename logic, returns `Outcome`

**Dependencies:** `clap` (CLI), `kamadak-exif` (EXIF reading), `walkdir` (recursive traversal)
