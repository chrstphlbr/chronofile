use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use nom_exif::{Exif, ExifIter, ExifTag, MediaParser, MediaSource, TrackInfo, TrackInfoTag};
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
    let mut parser = MediaParser::new();
    let ms = MediaSource::file_path(path).ok()?;

    if ms.has_exif() {
        let iter: ExifIter = parser.parse(ms).ok()?;
        let exif: Exif = iter.into();
        let val = exif.get(ExifTag::DateTimeOriginal)?;
        let (ndt, _) = val.as_time_components()?;
        Some(ndt.format("%Y%m%d").to_string())
    } else if ms.has_track() {
        let info: TrackInfo = parser.parse(ms).ok()?;
        let val = info.get(TrackInfoTag::CreateDate)?;
        let (ndt, _) = val.as_time_components()?;
        Some(ndt.format("%Y%m%d").to_string())
    } else {
        None
    }
}

fn already_prefixed(filename: &str) -> bool {
    let bytes = filename.as_bytes();
    bytes.len() > 9
        && bytes[..8].iter().all(|b| b.is_ascii_digit())
        && bytes[8] == b'-'
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
            }
        }
    };

    let new_name = format!("{}-{}", date, filename);
    let new_path = path.with_file_name(&new_name);

    if new_path.exists() {
        return Outcome::Skipped {
            path: path.to_path_buf(),
            reason: SkipReason::TargetExists,
        };
    }

    match fs::rename(path, &new_path) {
        Ok(()) => Outcome::Renamed {
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
                .map(|ext| matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "mov" | "mp4" | "m4v"))
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
