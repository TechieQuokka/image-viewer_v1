pub mod file;
pub mod sevenz_source;
pub mod zip_source;

use anyhow::Result;

/// All implementations hold only PathBuf/Vec<String>/String, so Send + Sync is safe.
pub trait ImageSource: Send + Sync {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn name_at(&self, index: usize) -> &str;
    fn load(&self, index: usize) -> Result<image::DynamicImage>;
    fn source_label(&self) -> &str;
    fn source_path(&self) -> &std::path::Path;
}

/// Open the appropriate ImageSource for the given path.
pub fn open_source(path: &std::path::Path) -> Result<Box<dyn ImageSource>> {
    use crate::utils::{detect_source_type, SourceType};
    match detect_source_type(path) {
        SourceType::FileSystem => {
            let src = file::FileSystemSource::open(path)?;
            Ok(Box::new(src))
        }
        SourceType::Zip => {
            let src = zip_source::ZipSource::open(path)?;
            Ok(Box::new(src))
        }
        SourceType::SevenZ => {
            let src = sevenz_source::SevenZSource::open(path)?;
            Ok(Box::new(src))
        }
        SourceType::Unsupported(msg) => Err(anyhow::anyhow!(msg)),
    }
}
