# chronofile

Copies JPEG and video files into `photos/` or `videos/` subfolders, prefixed with their capture/creation date. Originals are never modified.

```
photo.jpg   →  photos/20241231-photo.jpg
video.mov   →  videos/20241231-video.mov
```

## Requirements

- [exiftool](https://exiftool.org/) — required for video date extraction

```bash
brew install exiftool
```

## Installation

```bash
cargo build --release
# Binary is at ./target/release/chronofile
```

## Usage

```
chronofile <FOLDER>
```

Recursively scans `<FOLDER>` for `.jpg`, `.jpeg`, `.mov`, `.mp4`, and `.m4v` files (case-insensitive) and copies each one to a `photos/` or `videos/` subfolder (relative to the file's location) with a `YYYYMMDD-` date prefix. For photos, uses the `DateTimeOriginal` EXIF tag; for videos, uses `exiftool` to read `ContentCreateDate`, `DateTimeOriginal`, or `CreateDate` (in priority order). Original files are left untouched.

## Behavior

- **Copied** — printed to stdout: `Renamed: old/path.jpg -> old/photos/20241231-path.jpg`
- **Skipped** — printed to stderr with a reason:
  - `already prefixed` — filename already starts with `YYYYMMDD-`
  - `no EXIF date` — file has no readable date metadata
  - `target already exists` — a file with the new name already exists in the subfolder
  - `I/O error: ...` — filesystem error during copy
- A summary line is printed at the end: `Renamed: 3, Skipped: 1`

Files without readable date metadata (screenshots, downloaded images, etc.) are skipped. All originals are preserved.
