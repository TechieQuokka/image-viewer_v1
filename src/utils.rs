use std::path::Path;

/// Check if a path points to a supported image file
pub fn is_image_path(path: &Path) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(
            ext.to_lowercase().as_str(),
            "jpg" | "jpeg" | "png" | "gif" | "bmp" | "webp" | "tiff" | "tif"
        ),
        None => false,
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum NaturalSortToken {
    Text(String),
    Num(u64),
}

/// Natural sort key: splits string into alternating text/number tokens.
pub fn natural_sort_key(s: &str) -> Vec<NaturalSortToken> {
    let mut tokens = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            let mut num_str = String::new();
            while let Some(&d) = chars.peek() {
                if d.is_ascii_digit() {
                    num_str.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(NaturalSortToken::Num(num_str.parse::<u64>().unwrap_or(0)));
        } else {
            let mut text = String::new();
            while let Some(&d) = chars.peek() {
                if !d.is_ascii_digit() {
                    text.push(d);
                    chars.next();
                } else {
                    break;
                }
            }
            tokens.push(NaturalSortToken::Text(text.to_lowercase()));
        }
    }
    tokens
}

pub fn natural_sort(paths: &mut Vec<std::path::PathBuf>) {
    paths.sort_by(|a, b| {
        let ka = natural_sort_key(a.to_string_lossy().as_ref());
        let kb = natural_sort_key(b.to_string_lossy().as_ref());
        ka.cmp(&kb)
    });
}

pub enum SourceType {
    FileSystem,
    Zip,
    SevenZ,
    Unsupported(String),
}

/// Returns the 1-based position and total count of `current` among its sibling sources.
pub fn sibling_position(current: &Path) -> Option<(usize, usize)> {
    let parent = current.parent()?;
    let is_archive = matches!(
        detect_source_type(current),
        SourceType::Zip | SourceType::SevenZ
    );

    let mut siblings: Vec<std::path::PathBuf> = std::fs::read_dir(parent)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            if is_archive {
                matches!(detect_source_type(p), SourceType::Zip | SourceType::SevenZ)
            } else {
                p.is_dir()
            }
        })
        .collect();

    if siblings.len() <= 1 {
        return None;
    }

    natural_sort(&mut siblings);
    let pos = siblings.iter().position(|p| p == current)?;
    Some((pos + 1, siblings.len()))
}

pub fn sibling_source_path(current: &Path, forward: bool) -> Option<std::path::PathBuf> {
    let parent = current.parent()?;
    let is_archive = matches!(
        detect_source_type(current),
        SourceType::Zip | SourceType::SevenZ
    );

    let mut siblings: Vec<std::path::PathBuf> = std::fs::read_dir(parent)
        .ok()?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            if is_archive {
                matches!(detect_source_type(p), SourceType::Zip | SourceType::SevenZ)
            } else {
                p.is_dir()
            }
        })
        .collect();

    natural_sort(&mut siblings);

    let pos = siblings.iter().position(|p| p == current)?;
    if forward {
        siblings.into_iter().nth(pos + 1)
    } else {
        if pos == 0 {
            None
        } else {
            siblings.into_iter().nth(pos - 1)
        }
    }
}

pub fn detect_source_type(path: &Path) -> SourceType {
    if path.is_dir() {
        return SourceType::FileSystem;
    }
    if is_image_path(path) {
        return SourceType::FileSystem;
    }
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => match ext.to_lowercase().as_str() {
            "zip" | "cbz" => SourceType::Zip,
            "7z" | "cb7" => SourceType::SevenZ,
            "rar" | "cbr" => SourceType::Unsupported(
                "RAR format is not supported. Please convert to ZIP or 7Z.".into(),
            ),
            e => SourceType::Unsupported(format!("Unknown file type: .{}", e)),
        },
        None => SourceType::Unsupported("No file extension detected".into()),
    }
}
