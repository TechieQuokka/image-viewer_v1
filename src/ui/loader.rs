use crate::image_loader::ImageSource;
use std::sync::{mpsc, Arc};

// ── Message types ─────────────────────────────────────────────────────────────

struct LoadRequest {
    index: usize,
    source_id: u64,
    source: Arc<dyn ImageSource>,
}

pub struct LoadResult {
    pub index: usize,
    pub source_id: u64,
    pub image: Result<image::DynamicImage, String>,
}

// ── LoaderHandle ──────────────────────────────────────────────────────────────

/// Manages two background threads: one for the current image (primary) and
/// one for prefetch requests. Dropping this handle shuts down both threads.
pub struct LoaderHandle {
    primary_tx: mpsc::Sender<LoadRequest>,
    prefetch_tx: mpsc::Sender<LoadRequest>,
}

fn worker(rx: mpsc::Receiver<LoadRequest>, tx: mpsc::Sender<LoadResult>) {
    while let Ok(req) = rx.recv() {
        let image = req.source.load(req.index).map_err(|e| e.to_string());
        if tx
            .send(LoadResult {
                index: req.index,
                source_id: req.source_id,
                image,
            })
            .is_err()
        {
            break; // receiver dropped → window closed
        }
    }
}

impl LoaderHandle {
    /// Spawns two background threads. Results are sent back via `result_tx`.
    pub fn new(result_tx: mpsc::Sender<LoadResult>) -> Self {
        let (primary_tx, primary_rx) = mpsc::channel::<LoadRequest>();
        let (prefetch_tx, prefetch_rx) = mpsc::channel::<LoadRequest>();

        let tx2 = result_tx.clone();
        std::thread::spawn(move || worker(primary_rx, result_tx));
        std::thread::spawn(move || worker(prefetch_rx, tx2));

        Self { primary_tx, prefetch_tx }
    }

    /// Enqueue a primary (current image) load request.
    pub fn request_primary(&self, index: usize, source_id: u64, source: Arc<dyn ImageSource>) {
        let _ = self.primary_tx.send(LoadRequest { index, source_id, source });
    }

    /// Enqueue a prefetch load request.
    pub fn request_prefetch(&self, index: usize, source_id: u64, source: Arc<dyn ImageSource>) {
        let _ = self.prefetch_tx.send(LoadRequest { index, source_id, source });
    }
}
