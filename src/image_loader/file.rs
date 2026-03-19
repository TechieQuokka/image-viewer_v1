use super::ImageSource;
use crate::utils::{is_image_path, natural_sort};
use anyhow::{Context, Result};
use std::path::PathBuf;

pub struct FileSystemSource {
    dir: PathBuf,
    files: Vec<PathBuf>,
    label: String,
    current_index: usize,
}

impl FileSystemSource {
    /// Open a file or directory. If `path` is an image file, scans its parent directory.
    pub fn open(path: &std::path::Path) -> Result<Self> {
        let (dir, start_file) = if path.is_dir() {
            (path.to_path_buf(), None)
        } else {
            let parent = path
                .parent()
                .context("No parent directory")?
                .to_path_buf();
            (parent, Some(path.to_path_buf()))
        };

        let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
            .with_context(|| format!("Cannot read directory: {}", dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.is_file() && is_image_path(p))
            .collect();

        natural_sort(&mut files);

        let label = dir.to_string_lossy().into_owned();

        let current_index = start_file
            .and_then(|f| files.iter().position(|p| p == &f))
            .unwrap_or(0);

        Ok(Self {
            dir,
            files,
            label,
            current_index,
        })
    }

    pub fn start_index(&self) -> usize {
        self.current_index
    }
}

impl ImageSource for FileSystemSource {
    fn len(&self) -> usize {
        self.files.len()
    }

    fn name_at(&self, index: usize) -> &str {
        self.files[index]
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
    }

    fn load(&self, index: usize) -> Result<image::DynamicImage> {
        let path = &self.files[index];
        image::open(path).with_context(|| format!("Failed to open image: {}", path.display()))
    }

    fn source_label(&self) -> &str {
        &self.label
    }

    fn source_path(&self) -> &std::path::Path {
        &self.dir
    }
}
