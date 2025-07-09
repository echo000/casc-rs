#[cfg(target_os = "windows")]
fn main() {
    use embed_manifest::{
        embed_manifest,
        manifest::{ActiveCodePage, DpiAwareness, HeapType, Setting},
        new_manifest,
    };

    // Manifest
    embed_manifest(
        new_manifest("CascViewer.manifest")
            .active_code_page(ActiveCodePage::Utf8)
            .dpi_awareness(DpiAwareness::PerMonitorV2)
            .heap_type(HeapType::SegmentHeap)
            .long_path_aware(Setting::Enabled)
            .ui_access(false),
    )
    .expect("unable to embed manifest file");

    // Resource
    winresource::WindowsResource::new()
        /*.set_icon("res/Saluki.ico")*/
        .set_language(0x0409)
        .compile()
        .expect("unable to compile Windows resource");

    // Add the link search path
    println!("cargo:rustc-link-search=native=lib");

    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
