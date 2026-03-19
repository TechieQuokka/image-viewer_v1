use gtk4::prelude::*;
use gtk4::{Align, Label};

pub fn build_statusbar() -> Label {
    let label = Label::builder()
        .label("")
        .halign(Align::Start)
        .valign(Align::End)
        .margin_start(8)
        .margin_bottom(6)
        .build();

    label.add_css_class("statusbar");

    let provider = gtk4::CssProvider::new();
    // load_from_data works in all GTK 4.x versions (load_from_string requires v4_12)
    provider.load_from_data(
        ".statusbar { \
            background-color: rgba(0,0,0,0.55); \
            color: #ffffff; \
            padding: 2px 10px; \
            border-radius: 4px; \
            font-size: 0.85em; \
        }",
    );

    // Add provider to the default display (replaces deprecated style_context API)
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }

    label
}
