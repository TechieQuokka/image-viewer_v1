use crate::image_loader::ImageSource;
use crate::viewer::ViewerState;
use std::sync::Arc;

const CACHE_WINDOW: usize = 64;

pub enum SlotState {
    Empty,
    Loading,
    Ready(gtk4::gdk::MemoryTexture),
}

pub struct ImageMetadata {
    pub filename: String,
    pub width: u32,
    pub height: u32,
    pub index: usize,
    pub total: usize,
}

pub struct AppState {
    pub source: Option<Arc<dyn ImageSource>>,
    /// Monotonically increasing ID — incremented each time a new source is opened.
    pub source_id: u64,
    /// Per-image load state, indexed by image index.
    pub slots: Vec<SlotState>,
    pub current_index: usize,
    pub metadata: Option<ImageMetadata>,
    pub viewer: ViewerState,
    /// (1-based position, total) of the current source among its siblings.
    pub sibling_info: Option<(usize, usize)>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            source: None,
            source_id: 0,
            slots: Vec::new(),
            current_index: 0,
            metadata: None,
            viewer: ViewerState::default(),
            sibling_info: None,
        }
    }
}

impl AppState {
    /// Replace the current source. Resets all slots and increments source ID.
    pub fn set_source(&mut self, source: Arc<dyn ImageSource>, start_index: usize) {
        let len = source.len();
        self.source = Some(source);
        self.source_id += 1;
        self.current_index = start_index;
        self.slots = (0..len).map(|_| SlotState::Empty).collect();
        self.metadata = None;
        self.sibling_info = None;
        self.viewer.reset_pan();
    }

    pub fn slot_is_empty(&self, index: usize) -> bool {
        matches!(self.slots.get(index), Some(SlotState::Empty))
    }

    pub fn mark_loading(&mut self, index: usize) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = SlotState::Loading;
        }
    }

    pub fn store_texture(&mut self, index: usize, texture: gtk4::gdk::MemoryTexture) {
        if let Some(slot) = self.slots.get_mut(index) {
            *slot = SlotState::Ready(texture);
        }
    }

    /// Evict textures outside ±CACHE_WINDOW of current index back to Empty.
    pub fn evict_distant(&mut self) {
        let current = self.current_index;
        for (i, slot) in self.slots.iter_mut().enumerate() {
            let dist = if i >= current { i - current } else { current - i };
            if dist > CACHE_WINDOW {
                *slot = SlotState::Empty;
            }
        }
    }

    pub fn get_texture(&self, index: usize) -> Option<&gtk4::gdk::MemoryTexture> {
        match self.slots.get(index) {
            Some(SlotState::Ready(t)) => Some(t),
            _ => None,
        }
    }

    /// Return the GPU texture for the currently displayed image, if ready.
    pub fn current_texture(&self) -> Option<&gtk4::gdk::MemoryTexture> {
        self.get_texture(self.current_index)
    }

    /// Update viewer state and metadata when a new image is ready to display.
    pub fn update_display(&mut self, index: usize, width: u32, height: u32) {
        self.viewer.image_size = (width as f64, height as f64);
        self.viewer.reset_pan();
        let (filename, total) = if let Some(src) = &self.source {
            (src.name_at(index).to_owned(), src.len())
        } else {
            (String::new(), 0)
        };
        self.metadata = Some(ImageMetadata { filename, width, height, index, total });
    }

    pub fn status_text(&self) -> String {
        let zoom_pct = (self.viewer.effective_scale() * 100.0).round();
        match &self.metadata {
            Some(m) => {
                let sibling = match self.sibling_info {
                    Some((pos, total)) => format!("  [{}/{}]", pos, total),
                    None => String::new(),
                };
                format!(
                    "{}  {}×{}  {}%  [{}/{}]{}",
                    m.filename, m.width, m.height, zoom_pct, m.index + 1, m.total, sibling
                )
            }
            None => String::new(),
        }
    }
}

/// Convert an image::DynamicImage to a GDK MemoryTexture (RGBA, no premultiplication).
pub fn rgba_to_gdk_texture(img: &image::DynamicImage) -> Option<gtk4::gdk::MemoryTexture> {
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    let stride = (w * 4) as usize;
    let bytes = glib::Bytes::from_owned(rgba.into_raw());
    Some(gtk4::gdk::MemoryTexture::new(
        w as i32,
        h as i32,
        gtk4::gdk::MemoryFormat::R8g8b8a8,
        &bytes,
        stride,
    ))
}
