const STORAGE: &str = r#"F:\Call of Duty Modern Warfare"#;
const TEST_FILE: &str = "pak_base_vol021.xpak";
use casc_rs::casc_storage::CascStorage;

#[test]
fn test_load_file_name() {
    println!("Test started");
    let storage = CascStorage::new(STORAGE, None);
    assert!(storage.is_ok());
    println!("Storage loaded successfully");
    let unwrapped_storage = storage.unwrap();
    let file = unwrapped_storage.open_file_name(TEST_FILE);
    println!("File opened successfully");
    assert!(file.is_ok());
}
