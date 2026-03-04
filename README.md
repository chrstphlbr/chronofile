# macos-file-rename

Renames JPEG files in a directory by prefixing them with their EXIF capture date.

```
photo.jpg  →  20241231-photo.jpg
```

## Installation

```bash
cargo build --release
# Binary is at ./target/release/macos-file-rename
```

## Usage

```
macos-file-rename <FOLDER>
```

Recursively scans `<FOLDER>` for `.jpg`/`.jpeg` files (case-insensitive) and renames each one to `YYYYMMDD-<original_name>` using the `DateTimeOriginal` EXIF tag.

## Behavior

- **Renamed** — printed to stdout: `Renamed: old/path.jpg -> new/20241231-path.jpg`
- **Skipped** — printed to stderr with a reason:
  - `already prefixed` — filename already starts with `YYYYMMDD-`
  - `no EXIF date` — file has no `DateTimeOriginal` EXIF tag
  - `target already exists` — a file with the new name already exists
  - `I/O error: ...` — filesystem error during rename
- A summary line is printed at the end: `Renamed: 3, Skipped: 1`

Files without a readable EXIF date (screenshots, downloaded images, etc.) are silently skipped and never modified.
