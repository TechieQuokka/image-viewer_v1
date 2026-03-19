mod app;
mod image_loader;
mod ui;
mod utils;
mod viewer;

use gtk4::prelude::*;
use gtk4::Application;

const APP_ID: &str = "com.example.image-viewer-v1";

fn main() {
    env_logger::init();

    let app = Application::builder().application_id(APP_ID).build();

    // Set HANDLES_OPEN before signals so the app accepts file arguments
    app.set_flags(gtk4::gio::ApplicationFlags::HANDLES_OPEN);

    app.connect_activate(|app| {
        let window = ui::build_window(app);
        window.present();
    });

    app.connect_open(|app, files, _hint| {
        let path = files.first().and_then(|f| f.path());
        let window = ui::window::build_window_with_path(app, path.as_deref());
        window.present();
    });

    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
}
