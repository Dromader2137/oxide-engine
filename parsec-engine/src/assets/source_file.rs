//! [AssetSourceFileHandler] takes in files and gets all the information about
//! contained assets. For example an .obj file can contain multiple meshes.

use std::path::{Path, PathBuf};

use crate::assets::AssetDescription;

pub trait AssetSourceFileHandler {
    const EXTENSION: &'static str;
    fn extract_assets(filepath: &Path) -> Vec<AssetDescription>;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AssetSourceFile {
    filepath: PathBuf,
    asset_ids: Vec<u32>,
}
