//! # casc-viewer
//!
//! `casc-viewer` is a graphical application for browsing and exporting files from Blizzard CASC storages.
//! Built with [porter-lib](https://github.com/dtzxporter/porter-lib), it provides a user-friendly interface
//! for exploring game data archives.
//!
//! ## Features
//! - Open `.build.info` files and browse contained assets
//! - View file metadata (name, type, size, status)
//! - Export files to disk
//!
//! ## Usage
//! ```sh
//! cargo run -p casc-viewer
//! ```
//! Then open a `.build.info` file from a CASC storage directory.
//!
//! ## License
//! GPL-3.0
#![cfg_attr(
    all(target_os = "windows", not(debug_assertions),),
    windows_subsystem = "windows"
)]
use porter_ui::PorterColorPalette;
mod asset_manager;

fn main() {
    porter_ui::create_main(asset_manager::AssetManager::new())
        .version("0.0.1")
        .name("Casc Viewer")
        .description("Casc Library Explorer")
        .column("Name", 350, None)
        .column("Type", 100, None)
        .column("Status", 150, None)
        .column("Info", 250, Some(PorterColorPalette::asset_info()))
        .file_filter("Build Info (*.build.info)", vec!["build.info"])
        .raw_files_forcable(false)
        .preview(false)
        .run();
}
