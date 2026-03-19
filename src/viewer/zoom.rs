/// Zoom modes available in the viewer
#[derive(Debug, Clone, PartialEq)]
pub enum ZoomMode {
    /// 1:1 pixel mapping — the default
    ActualSize,
    /// Fit both dimensions within the canvas (letterbox)
    FitToWindow,
    /// Fit width, scroll vertically if needed
    FitToWidth,
    /// Fit height, scroll horizontally if needed
    FitToHeight,
    /// User-specified scale factor
    Custom(f64),
}

impl ZoomMode {
    pub const ZOOM_STEP: f64 = 1.25;
    pub const MIN_ZOOM: f64 = 0.05;
    pub const MAX_ZOOM: f64 = 32.0;

    /// Compute the effective scale factor given canvas and image dimensions.
    pub fn effective_scale(
        &self,
        canvas_w: f64,
        canvas_h: f64,
        img_w: f64,
        img_h: f64,
    ) -> f64 {
        if img_w <= 0.0 || img_h <= 0.0 {
            return 1.0;
        }
        match self {
            ZoomMode::ActualSize => 1.0,
            ZoomMode::FitToWindow => {
                let sw = canvas_w / img_w;
                let sh = canvas_h / img_h;
                sw.min(sh).min(1.0) // never upscale in fit mode (matches HoneyView default)
            }
            ZoomMode::FitToWidth => canvas_w / img_w,
            ZoomMode::FitToHeight => canvas_h / img_h,
            ZoomMode::Custom(s) => s.clamp(Self::MIN_ZOOM, Self::MAX_ZOOM),
        }
    }

    /// Step zoom in, returning a Custom mode with the new scale.
    pub fn zoom_in(&self, canvas_w: f64, canvas_h: f64, img_w: f64, img_h: f64) -> ZoomMode {
        let current = self.effective_scale(canvas_w, canvas_h, img_w, img_h);
        let next = (current * Self::ZOOM_STEP).min(Self::MAX_ZOOM);
        ZoomMode::Custom(next)
    }

    /// Step zoom out, returning a Custom mode with the new scale.
    pub fn zoom_out(&self, canvas_w: f64, canvas_h: f64, img_w: f64, img_h: f64) -> ZoomMode {
        let current = self.effective_scale(canvas_w, canvas_h, img_w, img_h);
        let next = (current / Self::ZOOM_STEP).max(Self::MIN_ZOOM);
        ZoomMode::Custom(next)
    }

    /// Zoom toward a focus point (e.g. mouse cursor) and return the new pan offset.
    pub fn zoom_toward(
        &self,
        delta: f64,
        focus_x: f64,
        focus_y: f64,
        canvas_w: f64,
        canvas_h: f64,
        img_w: f64,
        img_h: f64,
        pan_x: f64,
        pan_y: f64,
    ) -> (ZoomMode, f64, f64) {
        let old_scale = self.effective_scale(canvas_w, canvas_h, img_w, img_h);
        let factor = if delta > 0.0 {
            Self::ZOOM_STEP
        } else {
            1.0 / Self::ZOOM_STEP
        };
        let new_scale = (old_scale * factor).clamp(Self::MIN_ZOOM, Self::MAX_ZOOM);

        // Keep the point under cursor fixed.
        // X: centered anchor,  Y: top-left anchor
        let old_ox = (canvas_w - img_w * old_scale) / 2.0 + pan_x;
        let old_oy = pan_y;

        let img_px = (focus_x - old_ox) / old_scale;
        let img_py = (focus_y - old_oy) / old_scale;

        let new_pan_x = (focus_x - img_px * new_scale) - (canvas_w - img_w * new_scale) / 2.0;
        let new_pan_y = focus_y - img_py * new_scale;

        (ZoomMode::Custom(new_scale), new_pan_x, new_pan_y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_size_is_one() {
        assert_eq!(ZoomMode::ActualSize.effective_scale(800.0, 600.0, 400.0, 300.0), 1.0);
    }

    #[test]
    fn fit_to_window_scales_down() {
        // 2000x1000 image in 800x600 canvas → scale = min(0.4, 0.6) = 0.4
        let scale = ZoomMode::FitToWindow.effective_scale(800.0, 600.0, 2000.0, 1000.0);
        assert!((scale - 0.4).abs() < 1e-9);
    }

    #[test]
    fn fit_to_window_no_upscale() {
        // small image in large canvas → scale stays ≤ 1
        let scale = ZoomMode::FitToWindow.effective_scale(800.0, 600.0, 100.0, 100.0);
        assert!(scale <= 1.0);
    }
}
