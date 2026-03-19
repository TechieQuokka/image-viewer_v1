use super::ImageSource;
use crate::utils::is_image_path;
use anyhow::{Context, Result};
use std::io::Read;
use std::path::{Path, PathBuf};

pub struct ZipSource {
    path: PathBuf,
    entries: Vec<String>,
    label: String,
}

impl ZipSource {
    pub fn open(path: &Path) -> Result<Self> {
        let label = path.to_string_lossy().into_owned();
        let file = std::fs::File::open(path)
            .with_context(|| format!("Cannot open ZIP: {}", path.display()))?;
        let mut archive = zip::ZipArchive::new(file)?;

        let mut entries: Vec<String> = (0..archive.len())
            .filter_map(|i| {
                let entry = archive.by_index(i).ok()?;
                let name = entry.name().to_owned();
                let p = Path::new(&name);
                if is_image_path(p) && !entry.is_dir() {
                    Some(name)
                } else {
                    None
                }
            })
            .collect();

        entries.sort_by(|a, b| {
            use crate::utils::natural_sort_key;
            natural_sort_key(a).cmp(&natural_sort_key(b))
        });

        Ok(Self {
            path: path.to_path_buf(),
            entries,
            label,
        })
    }
}

impl ImageSource for ZipSource {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn name_at(&self, index: usize) -> &str {
        &self.entries[index]
    }

    fn load(&self, index: usize) -> Result<image::DynamicImage> {
        let file = std::fs::File::open(&self.path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let name = &self.entries[index];
        let mut entry = archive
            .by_name(name)
            .with_context(|| format!("Entry not found in ZIP: {}", name))?;

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        image::load_from_memory(&buf).context("Failed to decode image from ZIP")
    }

    fn source_label(&self) -> &str {
        &self.label
    }

    fn source_path(&self) -> &std::path::Path {
        &self.path
    }
}
