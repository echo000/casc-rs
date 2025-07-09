//! # casc-rs
//!
//! `casc-rs` is a pure Rust implementation of a Casc Storage Handler for Blizzard's CASC format.
//! It enables reading, listing, and extracting files from Blizzard game data archives.
//!
//! ## Features
//! - Read and parse CASC storages (WoW, Overwatch, etc.)
//! - List files and their metadata
//! - Extract files by name or key
//! - No external dependencies (pure Rust, except for compression)
//!
//! ## Usage
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! casc-rs = "0.2"
//! ```
//!
//! ### Example: Listing and Extracting Files
//! ```rust
//! use casc_rs::casc_storage::CascStorage;
//! use std::fs::File;
//!
//! // Open a CASC storage directory (containing .build.info, config, Data/)
//! let storage = CascStorage::new("path/to/casc/storage", None).unwrap();
//!
//! // List all files
//! for file_info in &storage.files {
//!     println!("File: {} ({} bytes)", file_info.file_name, file_info.file_size);
//! }
//!
//! // Extract a file by name
//! let mut casc_stream = storage.open_file_name("some/file/in/storage.txt").unwrap();
//! let mut output = File::create("output.txt").unwrap();
//! std::io::copy(&mut casc_stream, &mut output).unwrap();
//! ```

#![allow(unused)]
mod block_table;
mod casc_build_info;
mod casc_config;
mod casc_file_frame;
mod casc_file_info;
mod casc_file_span;
pub mod casc_file_stream;
mod casc_key_mapping_table;
mod casc_span_header;
pub mod casc_storage;
mod entry;
mod ext;
mod path_table_node_flags;
mod span_info;
mod tvfs_root_handler;
mod utility;
