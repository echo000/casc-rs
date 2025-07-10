# casc-rs

A pure Rust implementation of a Casc Storage Handler, inspired by the version ported to C# from C++.
This crate allows you to read and extract files from Blizzard's CASC storage format.

> **Note:** This library currently only supports CASC storages that use the TVFS root file format.

## Crates

- **casc-rs**: The core library for reading CASC storages.
- **casc-viewer**: A GUI application for browsing and exporting files from CASC storages, built with [porter-lib](https://github.com/dtzxporter/porter-lib).

---

## Usage

### Add to your `Cargo.toml`

```toml
[dependencies]
casc-rs = 0.1
```

### Example: Listing and Extracting Files

```rust
use casc_rs::casc_storage::CascStorage;
use std::fs::File;
use std::io::Write;

fn main() {
    // Open a CASC storage directory (containing .build.info, Data/)
    let storage = CascStorage::open("path/to/casc/storage").unwrap();

    // List all files
    for file_info in &storage.files {
        println!("File: {} ({} bytes)", file_info.file_name, file_info.file_size);
    }

    // Extract a file by name
    let file_name = "some/file/in/storage.txt";
    let mut casc_reader = storage.open_file(file_name).unwrap();
    let mut output = File::create("output.txt").unwrap();
    std::io::copy(&mut casc_reader, &mut output).unwrap();
}
```

---

## casc-viewer

A GUI application for exploring and exporting files from CASC storages.

- To run:
  ```
  cargo run -p casc-viewer
  ```
- Open a `.build.info` file from a CASC storage directory to browse its contents.

---
