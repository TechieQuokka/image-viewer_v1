use gtk4::prelude::*;
use gtk4::{Button, HeaderBar, Separator};

pub struct ToolbarButtons {
    pub open: Button,
    pub actual_size: Button,
    pub fit_window: Button,
    pub fit_width: Button,
    pub fit_height: Button,
    pub zoom_in: Button,
    pub zoom_out: Button,
    pub fullscreen: Button,
}

pub fn build_headerbar() -> (HeaderBar, ToolbarButtons) {
    let bar = HeaderBar::new();

    let open = Button::with_label("Open");
    open.set_icon_name("document-open-symbolic");
    open.set_tooltip_text(Some("Open file (Ctrl+O)"));
    bar.pack_start(&open);

    let sep = Separator::new(gtk4::Orientation::Vertical);
    bar.pack_start(&sep);

    let actual_size = Button::with_label("1:1");
    actual_size.set_tooltip_text(Some("Actual size (1)"));
    bar.pack_start(&actual_size);

    let fit_window = Button::with_label("Fit");
    fit_window.set_tooltip_text(Some("Fit to window (2)"));
    bar.pack_start(&fit_window);

    let fit_width = Button::with_label("Width");
    fit_width.set_tooltip_text(Some("Fit to width (3)"));
    bar.pack_start(&fit_width);

    let fit_height = Button::with_label("Height");
    fit_height.set_tooltip_text(Some("Fit to height (4)"));
    bar.pack_start(&fit_height);

    let sep2 = Separator::new(gtk4::Orientation::Vertical);
    bar.pack_start(&sep2);

    let zoom_out = Button::with_label("−");
    zoom_out.set_tooltip_text(Some("Zoom out (-)"));
    bar.pack_start(&zoom_out);

    let zoom_in = Button::with_label("+");
    zoom_in.set_tooltip_text(Some("Zoom in (+)"));
    bar.pack_start(&zoom_in);

    let fullscreen = Button::with_label("⛶");
    fullscreen.set_tooltip_text(Some("Fullscreen (F)"));
    bar.pack_end(&fullscreen);

    let buttons = ToolbarButtons {
        open,
        actual_size,
        fit_window,
        fit_width,
        fit_height,
        zoom_in,
        zoom_out,
        fullscreen,
    };
    (bar, buttons)
}
