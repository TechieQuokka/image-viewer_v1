use crate::app::AppState;
use crate::image_loader::open_source;
use crate::utils::{sibling_position, sibling_source_path};
use crate::viewer::ZoomMode;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    ApplicationWindow, Box as GtkBox, EventControllerKey, Label, Orientation, Overlay,
};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::Arc;

use super::canvas::build_canvas;
use super::loader::{LoadResult, LoaderHandle};
use super::statusbar::build_statusbar;
use super::toolbar::build_headerbar;

pub fn build_window(app: &gtk4::Application) -> ApplicationWindow {
    build_window_with_path(app, None)
}

pub fn build_window_with_path(
    app: &gtk4::Application,
    initial_path: Option<&Path>,
) -> ApplicationWindow {
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Image Viewer")
        .default_width(900)
        .default_height(700)
        .build();

    let state: Rc<RefCell<AppState>> = Rc::new(RefCell::new(AppState::default()));

    // --- Layout ---
    let vbox = GtkBox::new(Orientation::Vertical, 0);
    let (headerbar, buttons) = build_headerbar();
    window.set_titlebar(Some(&headerbar));

    let overlay = Overlay::new();
    let canvas = build_canvas(state.clone());
    let statusbar = build_statusbar();

    overlay.set_child(Some(&canvas));
    overlay.add_overlay(&statusbar);
    vbox.append(&overlay);
    window.set_child(Some(&vbox));

    // --- Background loader ---
    let (result_tx, result_rx) = std::sync::mpsc::channel::<LoadResult>();
    let loader: Rc<LoaderHandle> = Rc::new(LoaderHandle::new(result_tx));

    // --- navigate_to: display index N (cache hit → instant, miss → background load) ---
    let navigate_to: Rc<dyn Fn(usize)> = {
        let state = state.clone();
        let loader = loader.clone();
        let canvas_c = canvas.clone();
        let statusbar_c = statusbar.clone();

        Rc::new(move |index: usize| {
            let (source, source_id) = {
                let st = state.borrow();
                (st.source.clone(), st.source_id)
            };
            let Some(source) = source else { return; };

            {
                let mut st = state.borrow_mut();
                st.current_index = index;
                st.evict_distant();
            }

            // Fast path: texture already ready
            let cached_dims = state.borrow()
                .get_texture(index)
                .map(|t| (t.width() as u32, t.height() as u32));

            if let Some((w, h)) = cached_dims {
                {
                    let mut st = state.borrow_mut();
                    st.update_display(index, w, h);
                }
                statusbar_c.set_label(&state.borrow().status_text());
                canvas_c.queue_draw();
                prefetch(&state, &loader, &source, source_id, index);
                return;
            }

            // Slow path: request background load
            state.borrow_mut().mark_loading(index);
            loader.request_primary(index, source_id, source.clone());
            prefetch(&state, &loader, &source, source_id, index);
        })
    };

    // --- Poll for load results at ~120 fps (8 ms interval) ---
    {
        let state = state.clone();
        let canvas_c = canvas.clone();
        let statusbar_c = statusbar.clone();
        let loader = loader.clone();

        glib::timeout_add_local(std::time::Duration::from_millis(8), move || {
            use std::sync::mpsc::TryRecvError;
            loop {
                match result_rx.try_recv() {
                    Ok(res) => {
                        on_load_result(res, &state, &canvas_c, &statusbar_c, &loader);
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => return glib::ControlFlow::Break,
                }
            }
            glib::ControlFlow::Continue
        });
    }

    // --- open_file: open a path, swap source, start loading ---
    let open_file = {
        let state = state.clone();
        let window_c = window.clone();
        let navigate_to = navigate_to.clone();

        Rc::new(move |path: PathBuf| {
            match open_source(&path) {
                Ok(src) => {
                    let start_index = if crate::utils::is_image_path(&path) && path.is_file() {
                        crate::image_loader::file::FileSystemSource::open(&path)
                            .map(|fs| fs.start_index())
                            .unwrap_or(0)
                    } else {
                        0
                    };

                    window_c.set_title(Some(&format!(
                        "Image Viewer — {}",
                        path.file_name().and_then(|n| n.to_str()).unwrap_or("")
                    )));

                    let source_arc: Arc<dyn crate::image_loader::ImageSource> = Arc::from(src);
                    state.borrow_mut().set_source(source_arc, start_index);
                    let sibling = state.borrow().source.as_ref()
                        .map(|s| s.source_path().to_path_buf())
                        .and_then(|p| sibling_position(&p));
                    state.borrow_mut().sibling_info = sibling;
                    navigate_to(start_index);
                }
                Err(e) => show_error(&window_c, &e.to_string()),
            }
        })
    };

    // --- Open button ---
    {
        let open_file = open_file.clone();
        let window_c = window.clone();
        buttons.open.connect_clicked(move |_| {
            show_open_dialog(&window_c, open_file.clone());
        });
    }

    // --- Zoom / view mode buttons ---
    connect_zoom_button(&buttons.actual_size, ZoomMode::ActualSize, &state, &canvas, &statusbar);
    connect_zoom_button(&buttons.fit_window,  ZoomMode::FitToWindow, &state, &canvas, &statusbar);
    connect_zoom_button(&buttons.fit_width,   ZoomMode::FitToWidth,  &state, &canvas, &statusbar);
    connect_zoom_button(&buttons.fit_height,  ZoomMode::FitToHeight, &state, &canvas, &statusbar);

    {
        let state = state.clone(); let canvas_c = canvas.clone(); let statusbar_c = statusbar.clone();
        buttons.zoom_in.connect_clicked(move |_| {
            state.borrow_mut().viewer.zoom_in();
            statusbar_c.set_label(&state.borrow().status_text());
            canvas_c.queue_draw();
        });
    }
    {
        let state = state.clone(); let canvas_c = canvas.clone(); let statusbar_c = statusbar.clone();
        buttons.zoom_out.connect_clicked(move |_| {
            state.borrow_mut().viewer.zoom_out();
            statusbar_c.set_label(&state.borrow().status_text());
            canvas_c.queue_draw();
        });
    }
    {
        let window_c = window.clone();
        let state = state.clone();
        buttons.fullscreen.connect_clicked(move |_| {
            toggle_fullscreen(&window_c, &state);
        });
    }

    // --- Keyboard shortcuts ---
    {
        let key_ctrl = EventControllerKey::new();
        let state_k = state.clone();
        let canvas_k = canvas.clone();
        let statusbar_k = statusbar.clone();
        let window_k = window.clone();
        let navigate_to_k = navigate_to.clone();
        let open_file_k = open_file.clone();

        // Edge-navigation state: true = key was released after hitting the boundary.
        let down_edge: Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let up_edge:   Rc<RefCell<bool>> = Rc::new(RefCell::new(false));
        let down_edge_r = down_edge.clone();
        let up_edge_r   = up_edge.clone();
        key_ctrl.connect_key_released(move |_, keyval, _, _| {
            use gtk4::gdk::Key;
            match keyval {
                Key::Down | Key::KP_Down => *down_edge_r.borrow_mut() = true,
                Key::Up   | Key::KP_Up   => *up_edge_r.borrow_mut()   = true,
                _ => {}
            }
        });

        key_ctrl.connect_key_pressed(move |_ctrl, keyval, _code, mods| {
            use gtk4::gdk::Key;
            let ctrl = mods.contains(gtk4::gdk::ModifierType::CONTROL_MASK);

            let nav = |forward: bool| {
                use crate::viewer::navigation::{next_index, prev_index};
                let (total, current) = {
                    let st = state_k.borrow();
                    (st.source.as_ref().map(|s| s.len()).unwrap_or(0), st.current_index)
                };
                if total == 0 { return; }
                let idx = if forward { next_index(current, total) } else { prev_index(current, total) };
                navigate_to_k(idx);
            };

            let consumed = match keyval {
                Key::Page_Up | Key::KP_Page_Up => { nav(false); true }
                Key::Page_Down | Key::KP_Page_Down => { nav(true); true }
                Key::Home | Key::KP_Home => {
                    let total = state_k.borrow().source.as_ref().map(|s| s.len()).unwrap_or(0);
                    if total > 0 { navigate_to_k(0); }
                    true
                }
                Key::End | Key::KP_End => {
                    let total = state_k.borrow().source.as_ref().map(|s| s.len()).unwrap_or(0);
                    if total > 0 { navigate_to_k(total - 1); }
                    true
                }

                Key::Left | Key::KP_Left => {
                    let step = state_k.borrow().viewer.canvas_size.0 * 0.1;
                    { let mut st = state_k.borrow_mut(); st.viewer.pan_offset.0 += step; st.viewer.clamp_pan(); }
                    true
                }
                Key::Right | Key::KP_Right => {
                    let step = state_k.borrow().viewer.canvas_size.0 * 0.1;
                    { let mut st = state_k.borrow_mut(); st.viewer.pan_offset.0 -= step; st.viewer.clamp_pan(); }
                    true
                }
                Key::Up | Key::KP_Up => {
                    let step = state_k.borrow().viewer.canvas_size.1 * 0.1;
                    let at_top = {
                        let mut st = state_k.borrow_mut();
                        let before = st.viewer.pan_offset.1;
                        st.viewer.pan_offset.1 += step;
                        st.viewer.clamp_pan();
                        st.viewer.pan_offset.1 == before
                    };
                    if at_top {
                        let released = { let v = *up_edge.borrow(); v };
                        if released { nav(false); }
                        *up_edge.borrow_mut() = false;
                    } else {
                        *up_edge.borrow_mut() = false;
                    }
                    true
                }
                Key::Down | Key::KP_Down => {
                    let step = state_k.borrow().viewer.canvas_size.1 * 0.1;
                    let at_bottom = {
                        let mut st = state_k.borrow_mut();
                        let before = st.viewer.pan_offset.1;
                        st.viewer.pan_offset.1 -= step;
                        st.viewer.clamp_pan();
                        st.viewer.pan_offset.1 == before
                    };
                    if at_bottom {
                        let released = { let v = *down_edge.borrow(); v };
                        if released { nav(true); }
                        *down_edge.borrow_mut() = false;
                    } else {
                        *down_edge.borrow_mut() = false;
                    }
                    true
                }

                Key::plus | Key::KP_Add | Key::equal => { state_k.borrow_mut().viewer.zoom_in(); true }
                Key::minus | Key::KP_Subtract => { state_k.borrow_mut().viewer.zoom_out(); true }
                Key::_1 => { state_k.borrow_mut().viewer.set_zoom(ZoomMode::ActualSize); true }
                Key::_2 => { state_k.borrow_mut().viewer.set_zoom(ZoomMode::FitToWindow); true }
                Key::_3 => { state_k.borrow_mut().viewer.set_zoom(ZoomMode::FitToWidth); true }
                Key::_4 => { state_k.borrow_mut().viewer.set_zoom(ZoomMode::FitToHeight); true }

                Key::f | Key::F => { toggle_fullscreen(&window_k, &state_k); true }
                Key::Escape => {
                    window_k.unfullscreen();
                    state_k.borrow_mut().viewer.is_fullscreen = false;
                    true
                }
                Key::o | Key::O if ctrl => {
                    show_open_dialog(&window_k, open_file_k.clone());
                    true
                }
                Key::bracketleft => {
                    let path = state_k.borrow().source.as_ref()
                        .map(|s| s.source_path().to_path_buf());
                    if let Some(current) = path {
                        if let Some(prev) = sibling_source_path(&current, false) {
                            open_file_k(prev);
                        }
                    }
                    true
                }
                Key::bracketright => {
                    let path = state_k.borrow().source.as_ref()
                        .map(|s| s.source_path().to_path_buf());
                    if let Some(current) = path {
                        if let Some(next) = sibling_source_path(&current, true) {
                            open_file_k(next);
                        }
                    }
                    true
                }
                _ => false,
            };

            if consumed {
                statusbar_k.set_label(&state_k.borrow().status_text());
                canvas_k.queue_draw();
            }
            if consumed { glib::Propagation::Stop } else { glib::Propagation::Proceed }
        });

        window.add_controller(key_ctrl);
    }

    // --- Open initial path if provided ---
    if let Some(path) = initial_path {
        let path = path.to_path_buf();
        let open_file = open_file.clone();
        glib::idle_add_local_once(move || {
            open_file(path);
        });
    }

    window
}

// ── helpers ──────────────────────────────────────────────────────────────────

/// Called on the main thread each time a background load completes.
fn on_load_result(
    res: LoadResult,
    state: &Rc<RefCell<AppState>>,
    canvas: &impl WidgetExt,
    statusbar: &Label,
    loader: &Rc<LoaderHandle>,
) {
    let LoadResult { index, source_id, image } = res;

    if state.borrow().source_id != source_id {
        return;
    }

    match image {
        Ok(img) => {
            let w = img.width();
            let h = img.height();
            if let Some(texture) = crate::app::rgba_to_gdk_texture(&img) {
                let is_current = {
                    let mut st = state.borrow_mut();
                    st.store_texture(index, texture);
                    index == st.current_index
                };

                if is_current {
                    {
                        let mut st = state.borrow_mut();
                        st.update_display(index, w, h);
                    }
                    statusbar.set_label(&state.borrow().status_text());
                    canvas.queue_draw();
                }

                // Cascade: any completed load triggers prefetch of its neighbors.
                let (source, sid) = {
                    let st = state.borrow();
                    (st.source.clone(), st.source_id)
                };
                if let Some(src) = source {
                    prefetch(state, loader, &src, sid, index);
                }
            }
        }
        Err(e) => log::error!("Image load error: {}", e),
    }
}

/// Enqueue prefetch loads for ±5 neighbors of `index`.
/// Skips slots already Loading or Ready. Marks queued slots as Loading immediately
/// to prevent duplicate requests.
fn prefetch(
    state: &Rc<RefCell<AppState>>,
    loader: &Rc<LoaderHandle>,
    source: &Arc<dyn crate::image_loader::ImageSource>,
    source_id: u64,
    index: usize,
) {
    let total = source.len();
    let current = state.borrow().current_index;

    let within_window = |candidate: usize| -> bool {
        let dist = if candidate >= current { candidate - current } else { current - candidate };
        dist <= 64
    };

    for offset in 1..=5 {
        let candidates = [
            if index + offset < total { Some(index + offset) } else { None },
            if index >= offset { Some(index - offset) } else { None },
        ];
        for candidate in candidates.into_iter().flatten() {
            if !within_window(candidate) {
                continue;
            }
            let is_empty = state.borrow().slot_is_empty(candidate);
            if is_empty {
                state.borrow_mut().mark_loading(candidate);
                loader.request_prefetch(candidate, source_id, source.clone());
            }
        }
    }
}

fn connect_zoom_button<W>(
    btn: &gtk4::Button,
    mode: ZoomMode,
    state: &Rc<RefCell<AppState>>,
    canvas: &W,
    statusbar: &Label,
) where
    W: WidgetExt + Clone + 'static,
{
    let state = state.clone();
    let canvas = canvas.clone();
    let statusbar = statusbar.clone();
    btn.connect_clicked(move |_| {
        state.borrow_mut().viewer.set_zoom(mode.clone());
        statusbar.set_label(&state.borrow().status_text());
        canvas.queue_draw();
    });
}

fn toggle_fullscreen(window: &ApplicationWindow, state: &Rc<RefCell<AppState>>) {
    let fs = state.borrow().viewer.is_fullscreen;
    if fs {
        window.unfullscreen();
        state.borrow_mut().viewer.is_fullscreen = false;
    } else {
        window.fullscreen();
        state.borrow_mut().viewer.is_fullscreen = true;
    }
}

fn show_open_dialog(window: &ApplicationWindow, open_file: Rc<dyn Fn(PathBuf)>) {
    let dialog = gtk4::FileDialog::builder()
        .title("Open Image or Archive")
        .modal(true)
        .build();
    dialog.open(Some(window), gtk4::gio::Cancellable::NONE, move |result| {
        if let Ok(file) = result {
            if let Some(path) = file.path() {
                open_file(path);
            }
        }
    });
}

fn show_error(window: &ApplicationWindow, message: &str) {
    let dialog = gtk4::AlertDialog::builder()
        .message(message)
        .modal(true)
        .build();
    dialog.show(Some(window));
}
