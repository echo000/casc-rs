const STORAGE: &str = r#"F:\Call of Duty Modern Warfare"#;
const TEST_FILE: &str = "pak_base_vol021.xpak";
use casc_rs::casc_storage::CascStorage;

#[test]
fn test_load_file_name() {
    println!("Test started");
    let storage = CascStorage::open(STORAGE);
    assert!(storage.is_ok());
    println!("Storage loaded successfully");
    let unwrapped_storage = storage.unwrap();
    let file = unwrapped_storage.open_file(TEST_FILE);
    println!("File opened successfully");
    assert!(file.is_ok());
}

#[test]
fn test_export_file() {
    println!("Test started");
    let storage = CascStorage::open(STORAGE);
    assert!(storage.is_ok());
    println!("Storage loaded successfully");
    let unwrapped_storage = storage.unwrap();
    let file = unwrapped_storage.open_file(TEST_FILE);
    println!("File opened successfully");
    assert!(file.is_ok());

    // Write the file to xyz.dat in the current directory
    use std::fs::File as StdFile;
    use std::io::copy;

    let mut casc_stream = file.unwrap();
    let mut output = StdFile::create("xyz.dat").expect("Failed to create xyz.dat");
    let bytes_copied = copy(&mut casc_stream, &mut output).expect("Failed to write file");
    println!("Wrote {} bytes to xyz.dat", bytes_copied);
}
