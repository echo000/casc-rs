use casc_rs::{casc_storage::CascStorage, error::CascError};
use porter_app::{
    AssetPreview, AssetStatus, Color, Controller, SearchAsset, SearchTerm, Settings,
    palette::ASSET_TYPE_MODEL,
};
use porter_utils::{AsHumanBytes, AtomicCancel, AtomicProgress, AtomicSemaphore, ResultExt};
use rayon::prelude::*;
use std::{
    io::Read,
    path::Path,
    sync::{Arc, RwLock},
};

pub struct Asset {
    pub name: String,
    pub asset_type: String,
    pub asset_size: u64,
    pub status: AssetStatus,
}

impl Asset {
    pub fn search(&self) -> SearchAsset {
        SearchAsset::new(self.name.clone())
    }

    /// Returns the name of the asset
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn info(&self) -> String {
        self.asset_size.as_human_bytes().to_string()
    }
    pub fn type_name(&self) -> String {
        self.asset_type.clone()
    }
    /// Returns the color of the asset type
    pub fn color(&self) -> Color {
        ASSET_TYPE_MODEL
    }

    pub fn status(&self) -> &AssetStatus {
        &self.status
    }
}

pub struct AssetManager {
    search_assets: Arc<RwLock<Option<Vec<usize>>>>,
    loaded_assets: Arc<RwLock<Vec<Asset>>>,
    storage: Arc<RwLock<Option<CascStorage>>>,
    export_cancel: AtomicCancel,
    export_progress: AtomicProgress,
}

impl AssetManager {
    pub fn new() -> Self {
        Self {
            search_assets: Arc::new(RwLock::new(None)),
            loaded_assets: Arc::new(RwLock::new(Vec::new())),
            storage: Arc::new(RwLock::new(None)),
            export_cancel: AtomicCancel::new(),
            export_progress: AtomicProgress::new(),
        }
    }
}

impl porter_app::AssetManager for AssetManager {
    /// Whether or not the asset manager supports loading game files on disk.
    fn supports_files(&self) -> bool {
        true
    }

    /// Gets information about the specific asset, in the form of column data.
    fn assets_info(&self, index: usize) -> Vec<(String, Option<Color>)> {
        let search_lock = self.search_assets.read().unwrap();
        let loaded_assets_lock = self.loaded_assets.read().unwrap();

        // Change this when new porter_lib out
        let asset_index = if let Some(search) = search_lock.as_ref() {
            search.get(index).copied()
        } else {
            Some(index)
        };

        let Some(asset_index) = asset_index else {
            return vec![];
        };

        match loaded_assets_lock.get(asset_index) {
            Some(asset) => vec![
                (asset.name(), None),
                (asset.type_name().to_string(), Some(asset.color())),
                (asset.status().to_string(), Some(asset.status().color())),
                (asset.info(), None),
            ],
            None => vec![],
        }
    }

    /// The number of visible assets, whether they are search results, or just loaded.
    fn assets_visible(&self) -> usize {
        let search_lock = self.search_assets.read().unwrap();

        if let Some(indexes) = search_lock.as_ref() {
            indexes.len()
        } else {
            self.loaded_assets.read().unwrap().len()
        }
    }

    /// The total number of assets loaded.
    fn assets_total(&self) -> usize {
        self.loaded_assets.read().unwrap().len()
    }

    fn search(&self, search: Option<SearchTerm>) {
        let Some(search) = search else {
            *self.search_assets.write().unwrap() = None;
            return;
        };

        let loaded_assets = self.loaded_assets.read().unwrap();

        let results = loaded_assets
            .par_iter()
            .enumerate()
            .filter_map(|(index, asset)| {
                if search.matches(asset.search()) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();

        *self.search_assets.write().unwrap() = Some(results);
    }

    /// Loads the files into the asset manager.
    fn load_files(
        &self,
        _settings: Settings,
        files: Vec<std::path::PathBuf>,
    ) -> Result<(), String> {
        if files.is_empty() {
            return Err("No files provided.".to_string());
        }
        {
            *self.storage.write().unwrap() = None;
            *self.loaded_assets.write().unwrap() = Vec::new();
        }
        let file = files.first().unwrap();
        let storage = CascStorage::open(file.parent().unwrap())
            .map_err(|e| format!("Failed to open Casc Storage {e}"))?;

        let mut entries = Vec::new();
        for entry in &storage.files {
            let status = if !entry.is_local() {
                AssetStatus::NOT_SUPPORTED
            } else {
                AssetStatus::LOADED
            };
            let name = entry.file_name();
            let ext = match Path::new(&name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext.to_ascii_uppercase())
            {
                Some(ext) => ext,
                None => continue,
            };
            let asset = Asset {
                name: name.to_string(),
                asset_type: ext.to_string(),
                status,
                asset_size: entry.file_size() as u64,
            };
            entries.push(asset);
        }
        {
            *self.storage.write().unwrap() = Some(storage);
            *self.loaded_assets.write().unwrap() = entries;
        }
        Ok(())
    }

    fn load_game(&self, _settings: Settings) -> Result<(), String> {
        Err("How did you even get here?".into())
    }

    fn export(&self, settings: Settings, assets: Vec<usize>, controller: Controller) {
        self.export_progress.reset(assets.len());
        self.export_cancel.reset();

        let semaphore = AtomicSemaphore::new();

        let output_path = settings.output_directory();
        porter_threads::scope(|scope| {
            for asset in assets {
                let controller = controller.clone();
                let lock = semaphore.wait();
                let cloned_dir = output_path.clone();
                let storage = self.storage.clone();
                scope.spawn(move |_| {
                    let search = self.search_assets.read().unwrap();
                    let assets = self.loaded_assets.read().unwrap();
                    let asset_index = search
                        .as_ref()
                        .and_then(|s| s.get(asset).copied())
                        .unwrap_or(asset);
                    let asset = &assets[asset_index];
                    if !asset.status().is_available() || asset.asset_size == 0 {
                        self.export_progress.increment();
                        controller.progress_update(false, self.export_progress.progress());
                        return drop(lock);
                    }
                    asset.status().set(AssetStatus::EXPORTING);
                    let _ = std::fs::create_dir_all(&cloned_dir)
                        .debug_fn(|e| format!("Failed to create dest dir `{cloned_dir:?}`: {e}"));

                    let storage_guard = storage.read().unwrap();
                    let Some(storage_ref) = storage_guard.as_ref() else {
                        // Handle None case
                        return;
                    };
                    let result = storage_ref.open_file(&asset.name).and_then(|mut file| {
                        let output_file_path = cloned_dir.join(&asset.name);
                        if let Some(parent) = output_file_path.parent() {
                            if let Err(e) = std::fs::create_dir_all(parent) {
                                return Err(CascError::Other(format!(
                                    "Failed to create output directory: {e}"
                                )));
                            }
                        }
                        let mut output_file = match std::fs::File::create(&output_file_path) {
                            Ok(f) => f,
                            Err(e) => {
                                return Err(CascError::Other(format!(
                                    "Failed to create output file: {e}"
                                )));
                            }
                        };
                        if let Err(e) = std::io::copy(&mut file, &mut output_file) {
                            return Err(CascError::Other(format!("Failed to copy data: {e}")));
                        }
                        Ok(())
                    });
                    // Check the result of the export operation
                    match result {
                        // If export was successful, set the status to exported
                        Ok(_) => asset.status().set(AssetStatus::EXPORTED),
                        // If export failed, set the status to error
                        Err(e) => {
                            eprintln!("Error exporting `{}`: {e}", asset.name());
                            asset.status().set(AssetStatus::ERROR);
                        }
                    }
                    self.export_progress.increment();
                    controller.progress_update(false, self.export_progress.progress());
                    drop(lock);
                });
                if self.export_cancel.is_cancelled() {
                    return;
                }
            }
        });
        // Finalize export
        controller.progress_update(true, 100);
    }

    ///Not used, but required by the trait.
    fn preview(
        &self,
        settings: Settings,
        asset: usize,
        raw: bool,
        request_id: u64,
        controller: Controller,
    ) {
        let _ = raw;
        let assets_guard = self.loaded_assets.read().unwrap();
        let storage_guard = self.storage.read().unwrap();
        let Some(storage_ref) = storage_guard.as_ref() else {
            // Handle None case
            return;
        };

        let (asset_name, asset_ref) = {
            let search = self.search_assets.read().unwrap();
            let asset_index = search
                .as_ref()
                .and_then(|s| s.get(asset).copied())
                .unwrap_or(asset);
            let selected_asset = &assets_guard[asset_index];
            // Ignore the placeholder assets
            if !selected_asset.status().is_available() {
                return;
            }
            (selected_asset.name(), selected_asset)
        };

        let result = storage_ref.open_file(&asset_name).and_then(|mut file| {
            let mut buf = Vec::with_capacity(asset_ref.asset_size as usize);
            let _ = file.read_to_end(&mut buf)?;
            Ok(buf)
        });

        let preview_asset = result
            .map(|bytes| AssetPreview::RawFile(asset_name, bytes))
            .ok();

        if let Some(preview) = preview_asset {
            controller.preview_update(request_id, preview);
        }
    }

    /// Cancels an active export.
    fn export_cancel(&self) {
        self.export_cancel.cancel();
    }
}
