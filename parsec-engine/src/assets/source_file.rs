//! [AssetSourceFileHandler] takes in files and gets all the information about
//! contained assets. For example an .obj file can contain multiple meshes.

use std::{collections::BTreeSet, path::{Path, PathBuf}};

use crate::assets::InitialAssetDescription;

pub trait AssetSourceFileHandler {
    const EXTENSION: &'static str;
    fn extract_assets(filepath: &Path) -> Vec<InitialAssetDescription>;
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct AssetSourceFile {
    pub filepath: PathBuf,
    pub asset_ids: BTreeSet<usize>,
}
