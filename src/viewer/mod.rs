pub mod navigation;
pub mod zoom;

pub use zoom::ZoomMode;

#[derive(Debug, Clone)]
pub struct ViewerState {
    pub zoom: ZoomMode,
    pub pan_offset: (f64, f64),
    pub canvas_size: (f64, f64),
    pub image_size: (f64, f64),
    pub is_fullscreen: bool,
}

impl Default for ViewerState {
    fn default() -> Self {
        Self {
            zoom: ZoomMode::ActualSize,
            pan_offset: (0.0, 0.0),
            canvas_size: (800.0, 600.0),
            image_size: (0.0, 0.0),
            is_fullscreen: false,
        }
    }
}

impl ViewerState {
    pub fn effective_scale(&self) -> f64 {
        self.zoom.effective_scale(
            self.canvas_size.0,
            self.canvas_size.1,
            self.image_size.0,
            self.image_size.1,
        )
    }

    /// Compute the draw offset for the image on the canvas.
    /// Horizontally centered, vertically top-aligned.
    pub fn draw_offset(&self) -> (f64, f64) {
        let scale = self.effective_scale();
        let ox = (self.canvas_size.0 - self.image_size.0 * scale) / 2.0 + self.pan_offset.0;
        let oy = self.pan_offset.1;
        (ox, oy)
    }

    pub fn reset_pan(&mut self) {
        self.pan_offset = (0.0, 0.0);
    }

    pub fn set_zoom(&mut self, mode: ZoomMode) {
        self.zoom = mode;
        self.reset_pan();
    }

    pub fn clamp_pan(&mut self) {
        let scale = self.effective_scale();
        let img_w = self.image_size.0 * scale;
        let img_h = self.image_size.1 * scale;

        // X: centered — pan is relative to center
        if img_w <= self.canvas_size.0 {
            self.pan_offset.0 = 0.0;
        } else {
            let half = (img_w - self.canvas_size.0) / 2.0;
            self.pan_offset.0 = self.pan_offset.0.clamp(-half, half);
        }

        // Y: top-aligned — pan starts at 0 (top) and scrolls down
        if img_h <= self.canvas_size.1 {
            self.pan_offset.1 = 0.0;
        } else {
            let max_scroll = img_h - self.canvas_size.1;
            self.pan_offset.1 = self.pan_offset.1.clamp(-max_scroll, 0.0);
        }
    }

    pub fn zoom_in(&mut self) {
        let new_zoom = self.zoom.zoom_in(
            self.canvas_size.0,
            self.canvas_size.1,
            self.image_size.0,
            self.image_size.1,
        );
        self.zoom = new_zoom;
        self.clamp_pan();
    }

    pub fn zoom_out(&mut self) {
        let new_zoom = self.zoom.zoom_out(
            self.canvas_size.0,
            self.canvas_size.1,
            self.image_size.0,
            self.image_size.1,
        );
        self.zoom = new_zoom;
        self.clamp_pan();
    }

    pub fn zoom_toward(&mut self, delta: f64, focus_x: f64, focus_y: f64) {
        let (new_zoom, new_pan_x, new_pan_y) = self.zoom.zoom_toward(
            delta,
            focus_x,
            focus_y,
            self.canvas_size.0,
            self.canvas_size.1,
            self.image_size.0,
            self.image_size.1,
            self.pan_offset.0,
            self.pan_offset.1,
        );
        self.zoom = new_zoom;
        self.pan_offset = (new_pan_x, new_pan_y);
        self.clamp_pan();
    }
}
