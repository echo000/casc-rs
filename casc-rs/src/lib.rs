//! # casc-rs
//!
//! `casc-rs` is a pure Rust implementation of a Casc Storage Handler for Blizzard's CASC format.
//! It enables reading, listing, and extracting files from Blizzard game data archives.
//!
//! ## Features
//! - Read and parse CASC storages (WoW, Overwatch, etc.)
//! - List files and their metadata
//! - Extract files by name
//!
//! ## CascStorage
//! The main entry point for interacting with CASC archives is the [`CascStorage`](crate::casc_storage::CascStorage) struct. It provides methods to open a CASC storage directory, list available files, and extract file contents. `CascStorage` handles parsing the storage's metadata, configuration, and file tables, allowing you to work with Blizzard game data archives in a high-level, ergonomic way.
//!
//! Typical usage involves creating a `CascStorage` instance with the path to your storage directory, then using its methods to list or extract files. See the example below for a typical workflow.
//!
//! ## File
//! The [`CascFile`](crate::file::CascFile) struct provides stream-like, read and seek access to the contents of files stored in a CASC archive. It is returned by methods such as `CascStorage::open_file_name` and implements the standard `Read` and `Seek` traits, allowing you to process file data just like with `std::fs::File`. This makes it easy to extract, process, or copy file contents from the archive in an idiomatic Rust way.
//!
//! ## Error Handling
//! All fallible operations in this crate return a [`CascError`](crate::error::CascError) type, which provides detailed information about possible errors such as file not found, invalid data, unsupported file types, I/O errors, and more. You can use standard Rust error handling patterns (`?`, `match`, etc.) to work with these errors.
//!
//! ## Usage
//! Add to your `Cargo.toml`:
//! ```toml
//! [dependencies]
//! casc-rs = "0.1"
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
pub mod casc_file;
mod casc_file_frame;
mod casc_file_info;
mod casc_file_span;
mod casc_key_mapping_table;
mod casc_span_header;
pub mod casc_storage;
mod entry;
pub mod error;
mod ext;
mod path_table_node_flags;
mod span_info;
mod tvfs_root_handler;
mod utility;
