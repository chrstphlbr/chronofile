# macos-file-rename

Renames JPEG and video files in a directory by prefixing them with their capture/creation date.

```
photo.jpg   →  20241231-photo.jpg
video.mov   →  20241231-video.mov
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

Recursively scans `<FOLDER>` for `.jpg`, `.jpeg`, `.mov`, `.mp4`, and `.m4v` files (case-insensitive) and renames each one to `YYYYMMDD-<original_name>`. For photos, uses the `DateTimeOriginal` EXIF tag; for videos, uses `exiftool` to read `ContentCreateDate`, `DateTimeOriginal`, or `CreateDate` (in priority order).

## Behavior

- **Renamed** — printed to stdout: `Renamed: old/path.jpg -> new/20241231-path.jpg`
- **Skipped** — printed to stderr with a reason:
  - `already prefixed` — filename already starts with `YYYYMMDD-`
  - `no EXIF date` — file has no `DateTimeOriginal` EXIF tag
  - `target already exists` — a file with the new name already exists
  - `I/O error: ...` — filesystem error during rename
- A summary line is printed at the end: `Renamed: 3, Skipped: 1`

Files without readable date metadata (screenshots, downloaded images, etc.) are silently skipped and never modified.
