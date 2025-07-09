use casc_rs::{casc_file_stream::CascFileStream, casc_storage::CascStorage};
use porter_ui::{
    Color, PorterAssetManager, PorterAssetStatus, PorterColorPalette, PorterSearch,
    PorterSearchAsset,
};
use porter_utils::{AsHumanBytes, AtomicCancel, AtomicProgress};
use rayon::prelude::*;
use std::{
    io,
    path::Path,
    sync::{Arc, RwLock},
};

pub struct Asset {
    pub name: String,
    pub asset_type: String,
    pub asset_size: u64,
    pub status: PorterAssetStatus,
}

impl Asset {
    pub fn search(&self) -> PorterSearchAsset {
        PorterSearchAsset::new(self.name.clone())
    }

    /// Returns the name of the asset
    pub fn name(&self) -> String {
        self.name.clone()
    }
    pub fn info(&self) -> String {
        format!("{}", self.asset_size.as_human_bytes())
    }
    pub fn type_name(&self) -> String {
        self.asset_type.clone()
    }
    /// Returns the color of the asset type
    pub fn color(&self) -> Color {
        PorterColorPalette::asset_type_model()
    }

    pub fn status(&self) -> &PorterAssetStatus {
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

impl PorterAssetManager for AssetManager {
    /// Returns the asset info in the form of the columns to render.
    fn asset_info(&self, index: usize, _columns: usize) -> Vec<(String, Option<Color>)> {
        let search = self.search_assets.read().unwrap();
        let loaded_assets = self.loaded_assets.read().unwrap();

        let asset_index = if let Some(search) = search.as_ref() {
            search.get(index).copied()
        } else {
            Some(index)
        };

        let Some(asset_index) = asset_index else {
            return vec![];
        };

        match loaded_assets.get(asset_index) {
            Some(asset) => vec![
                (asset.name(), None),
                (asset.type_name().to_string(), Some(asset.color())),
                (asset.status().to_string(), Some(asset.status().color())),
                (asset.info(), None),
            ],
            None => vec![],
        }
    }

    /// Returns the number of assets renderable, as in search for, or loaded.
    fn len(&self) -> usize {
        if let Some(indexes) = &*self.search_assets.read().unwrap() {
            indexes.len()
        } else {
            self.loaded_assets.read().unwrap().len()
        }
    }

    /// Returns the total number of assets loaded.
    fn loaded_len(&self) -> usize {
        self.loaded_assets.read().unwrap().len()
    }

    fn search_assets(&self, search: Option<PorterSearch>) {
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

    /// Whether load file is supported.
    fn supports_load_files(&self) -> bool {
        true
    }

    /// Whether load game is supported.
    fn supports_load_game(&self) -> bool {
        false
    }

    /// Loads the files into the asset manager.
    fn on_load_files(
        &self,
        _settings: porter_ui::PorterSettings,
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
        let storage = CascStorage::new(&file.parent().unwrap(), None)
            .map_err(|e| format!("Failed to open Casc Storage {e}"))?;

        let mut entries = Vec::new();
        for entry in &storage.files {
            if !entry.is_local {
                continue;
            }
            let name = entry.file_name.clone();
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
                status: PorterAssetStatus::loaded(),
                asset_size: entry.file_size as u64,
            };
            entries.push(asset);
        }
        {
            *self.storage.write().unwrap() = Some(storage);
            *self.loaded_assets.write().unwrap() = entries;
        }
        Ok(())
    }

    fn on_load_game(&self, _settings: porter_ui::PorterSettings) -> Result<(), String> {
        Err("How did you even get here?".into())
    }

    fn on_export(
        &self,
        settings: porter_ui::PorterSettings,
        assets: Vec<usize>,
        ui: porter_ui::PorterUI,
    ) {
        let output_path = settings.output_directory();
        self.export_progress.reset(assets.len());
        self.export_cancel.reset();
        let (search, all_assets, storage) = {
            let search = self.search_assets.read().unwrap().clone();
            let assets = self.loaded_assets.read().unwrap();
            let storage = self.storage.clone();
            (search, assets, storage)
        };
        assets.into_par_iter().for_each(|row| {
            if self.export_cancel.is_cancelled() {
                return;
            }
            let asset_index = search
                .as_ref()
                .and_then(|s| s.get(row).copied())
                .unwrap_or(row);

            let asset = &all_assets[asset_index];
            asset.status().set(PorterAssetStatus::exporting());

            if asset.asset_size <= 0 {
                self.export_progress.increment();
                asset.status().set(PorterAssetStatus::error());
                return;
            }

            let storage_guard = storage.read().unwrap();
            let Some(storage_ref) = storage_guard.as_ref() else {
                // Handle None case
                return;
            };
            let export_result = storage_ref
                .open_file_name(&asset.name)
                .and_then(|mut file| {
                    let output_file_path = output_path.join(&asset.name);
                    if let Some(parent) = output_file_path.parent() {
                        if let Err(e) = std::fs::create_dir_all(parent) {
                            return Err(io::Error::other(format!(
                                "Failed to create output directory: {e}"
                            )));
                        }
                    }
                    let mut output_file = match std::fs::File::create(&output_file_path) {
                        Ok(f) => f,
                        Err(e) => {
                            return Err(io::Error::other(format!(
                                "Failed to create output file: {e}"
                            )));
                        }
                    };
                    if let Err(e) = std::io::copy(&mut file, &mut output_file) {
                        return Err(io::Error::other(format!("Failed to copy data: {e}")));
                    }
                    asset.status().set(PorterAssetStatus::exported());
                    self.export_progress.increment();
                    Ok(())
                });

            if let Err(e) = export_result {
                asset.status().set(PorterAssetStatus::error());
                eprintln!("Error exporting {}: {}", asset.name, e);
            }
        });
        ui.sync(false, 100);
    }

    ///Not used, but required by the trait.
    fn on_preview(
        &self,
        _settings: porter_ui::PorterSettings,
        _asset: usize,
        _request_id: u64,
        _ui: porter_ui::PorterUI,
    ) {
        return;
    }

    /// Cancels an active export.
    fn cancel_export(&self) {
        self.export_cancel.cancel();
    }
}
