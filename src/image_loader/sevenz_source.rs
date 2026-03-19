use super::ImageSource;
use crate::utils::is_image_path;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

pub struct SevenZSource {
    path: PathBuf,
    entries: Vec<String>,
    label: String,
}

impl SevenZSource {
    pub fn open(path: &Path) -> Result<Self> {
        let label = path.to_string_lossy().into_owned();
        let mut entries: Vec<String> = Vec::new();

        sevenz_rust::decompress_file_with_extract_fn(
            path,
            Path::new(""),
            |entry, _reader, _dest| {
                let name = entry.name().to_owned();
                let p = Path::new(&name);
                if !entry.is_directory() && is_image_path(p) {
                    entries.push(name);
                }
                Ok(false) // don't extract
            },
        )
        .with_context(|| format!("Failed to read 7Z archive: {}", path.display()))?;

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

impl ImageSource for SevenZSource {
    fn len(&self) -> usize {
        self.entries.len()
    }

    fn name_at(&self, index: usize) -> &str {
        &self.entries[index]
    }

    fn load(&self, index: usize) -> Result<image::DynamicImage> {
        let target = self.entries[index].clone();
        let mut image_data: Option<Vec<u8>> = None;

        sevenz_rust::decompress_file_with_extract_fn(
            &self.path,
            Path::new(""),
            |entry, reader, _dest| {
                if entry.name() == target {
                    let mut data = Vec::new();
                    reader.read_to_end(&mut data)?;
                    image_data = Some(data);
                    Ok(true) // stop iterating
                } else {
                    Ok(false)
                }
            },
        )
        .with_context(|| format!("Failed to read 7Z archive: {}", self.path.display()))?;

        let data = image_data.context("Image entry not found in 7Z archive")?;
        image::load_from_memory(&data).context("Failed to decode image from 7Z")
    }

    fn source_label(&self) -> &str {
        &self.label
    }

    fn source_path(&self) -> &std::path::Path {
        &self.path
    }
}
