use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use nom_exif::{Exif, ExifIter, ExifTag, MediaParser, MediaSource};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(about = "Rename media files to YYYYMMDD-<original_name>")]
struct Args {
    folder: PathBuf,
}

enum Outcome {
    Renamed { from: PathBuf, to: PathBuf },
    Skipped { path: PathBuf, reason: SkipReason },
}

enum SkipReason {
    AlreadyPrefixed,
    NoExifDate,
    TargetExists,
    IoError(std::io::Error),
}

fn extract_date(path: &Path) -> Option<String> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    match ext.as_str() {
        "jpg" | "jpeg" => extract_date_exif(path),
        "mov" | "mp4" | "m4v" => extract_date_video(path),
        _ => None,
    }
}

fn extract_date_exif(path: &Path) -> Option<String> {
    let mut parser = MediaParser::new();
    let ms = MediaSource::file_path(path).ok()?;
    if ms.has_exif() {
        let iter: ExifIter = parser.parse(ms).ok()?;
        let exif: Exif = iter.into();
        let val = exif.get(ExifTag::DateTimeOriginal)?;
        let (ndt, _) = val.as_time_components()?;
        Some(ndt.format("%Y%m%d").to_string())
    } else {
        None
    }
}

fn extract_date_video(path: &Path) -> Option<String> {
    for tag in ["CreationDate", "DateTimeOriginal", "CreateDate"] {
        let output = std::process::Command::new("exiftool")
            .args([&format!("-{tag}"), "-s3", &path.to_string_lossy()])
            .output()
            .ok()?;
        let s = std::str::from_utf8(&output.stdout).ok()?.trim();
        if let Some(date) = parse_exiftool_date(s) {
            return Some(date);
        }
    }
    None
}

fn parse_exiftool_date(s: &str) -> Option<String> {
    if s.len() < 10 {
        return None;
    }
    let b = s.as_bytes();
    let sep = b[4];
    if (sep != b':' && sep != b'-') || b[7] != sep {
        return None;
    }
    let (year, month, day) = (&s[..4], &s[5..7], &s[8..10]);
    if [year, month, day]
        .iter()
        .all(|p| p.chars().all(|c| c.is_ascii_digit()))
    {
        Some(format!("{year}{month}{day}"))
    } else {
        None
    }
}

fn media_subfolder(path: &Path) -> &'static str {
    match path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase())
        .as_deref()
    {
        Some("jpg" | "jpeg") => "photos",
        _ => "videos",
    }
}

fn already_prefixed(filename: &str) -> bool {
    let bytes = filename.as_bytes();
    bytes.len() > 9 && bytes[..8].iter().all(|b| b.is_ascii_digit()) && bytes[8] == b'-'
}

fn process_file(path: &Path) -> Outcome {
    let filename = path.file_name().unwrap().to_string_lossy();

    if already_prefixed(&filename) {
        return Outcome::Skipped {
            path: path.to_path_buf(),
            reason: SkipReason::AlreadyPrefixed,
        };
    }

    let date = match extract_date(path) {
        Some(d) => d,
        None => {
            return Outcome::Skipped {
                path: path.to_path_buf(),
                reason: SkipReason::NoExifDate,
            };
        }
    };

    let new_name = format!("{}-{}", date, filename);
    let subfolder = path.parent().unwrap().join(media_subfolder(path));

    if let Err(e) = fs::create_dir_all(&subfolder) {
        return Outcome::Skipped {
            path: path.to_path_buf(),
            reason: SkipReason::IoError(e),
        };
    }

    let new_path = subfolder.join(&new_name);

    if new_path.exists() {
        return Outcome::Skipped {
            path: path.to_path_buf(),
            reason: SkipReason::TargetExists,
        };
    }

    match fs::copy(path, &new_path) {
        Ok(_) => Outcome::Renamed {
            from: path.to_path_buf(),
            to: new_path,
        },
        Err(e) => Outcome::Skipped {
            path: path.to_path_buf(),
            reason: SkipReason::IoError(e),
        },
    }
}

fn main() {
    let args = Args::parse();

    if !args.folder.is_dir() {
        eprintln!("Error: '{}' is not a directory", args.folder.display());
        std::process::exit(1);
    }

    if std::process::Command::new("exiftool")
        .arg("-ver")
        .output()
        .is_err()
    {
        eprintln!(
            "Warning: exiftool not found — video files will be skipped (install with: brew install exiftool)"
        );
    }

    let mut renamed = 0usize;
    let mut skipped = 0usize;

    for entry in WalkDir::new(&args.folder)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| {
                    matches!(
                        ext.to_lowercase().as_str(),
                        "jpg" | "jpeg" | "mov" | "mp4" | "m4v"
                    )
                })
                .unwrap_or(false)
        })
    {
        match process_file(entry.path()) {
            Outcome::Renamed { from, to } => {
                println!("Renamed: {} -> {}", from.display(), to.display());
                renamed += 1;
            }
            Outcome::Skipped { path, reason } => {
                let msg = match reason {
                    SkipReason::AlreadyPrefixed => "already prefixed".to_string(),
                    SkipReason::NoExifDate => "no EXIF date".to_string(),
                    SkipReason::TargetExists => "target already exists".to_string(),
                    SkipReason::IoError(e) => format!("I/O error: {e}"),
                };
                eprintln!("Skipped: {} ({})", path.display(), msg);
                skipped += 1;
            }
        }
    }

    println!("Renamed: {renamed}, Skipped: {skipped}");
}
