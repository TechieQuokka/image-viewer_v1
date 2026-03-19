use crate::app::AppState;
use std::cell::RefCell;
use std::rc::Rc;

pub fn build_canvas(state: Rc<RefCell<AppState>>) -> super::image_canvas::ImageCanvas {
    super::image_canvas::ImageCanvas::new(state)
}
