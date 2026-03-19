# image-viewer_v1

GTK4 image viewer written in Rust. Supports local directories and compressed archives (ZIP, 7Z).

## Features

- Open image files, directories, and archives (zip / cbz / 7z / cb7)
- Background image loading with ±5 prefetch
- Natural sort order for filenames
- Zoom: actual size, fit to window, fit to width, fit to height, custom
- Mouse wheel zoom toward cursor, drag to pan
- Edge-triggered navigation: scrolling past the top/bottom of an image moves to the previous/next image
- Sibling source navigation: jump to the previous/next directory or archive in the same parent folder
- Fullscreen mode

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `PageDown` / `PageUp` | Next / previous image |
| `Home` / `End` | First / last image |
| `↑` / `↓` | Scroll up / down (edge triggers image change) |
| `←` / `→` | Scroll left / right |
| `]` / `[` | Next / previous directory or archive |
| `+` / `-` | Zoom in / out |
| `1` | Actual size (1:1) |
| `2` | Fit to window |
| `3` | Fit to width |
| `4` | Fit to height |
| `F` | Toggle fullscreen |
| `Escape` | Exit fullscreen |
| `Ctrl+O` | Open file dialog |

## Supported Formats

| Type | Extensions |
|------|------------|
| Images | jpg, jpeg, png, gif, bmp, webp, tiff |
| ZIP archive | zip, cbz |
| 7Z archive | 7z, cb7 |

## Dependencies

- GTK 4.10+
- Rust edition 2024

## Build

```bash
cargo build --release
```

## Run

```bash
# Open a directory
cargo run --release -- /path/to/images/

# Open an archive
cargo run --release -- /path/to/archive.zip

# Open a specific image (starts at that image in its directory)
cargo run --release -- /path/to/image.jpg
```
