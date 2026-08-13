use std::{marker::PhantomData, path::PathBuf};

use crate::{assets::source_file::AssetSourceFile, ecs::resources::Resources};

pub mod core;
pub mod library;
pub mod manifest;
pub mod source_file;

#[derive(Debug, PartialEq, Eq)]
pub struct AssetHandle<T: Asset> {
    name: &'static str,
    _marker: PhantomData<T>,
}

impl<T: Asset> Clone for AssetHandle<T> {
    fn clone(&self) -> Self {
        Self {
            name: self.name,
            _marker: PhantomData,
        }
    }
}
impl<T: Asset> Copy for AssetHandle<T> {}
impl<T: Asset> AssetHandle<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            _marker: PhantomData,
        }
    }
}

pub trait Asset: 'static {
    type Cooked: serde::Serialize + serde::de::DeserializeOwned + 'static;

    const ASSET_TYPE: &'static str;

    fn cook(
        asset_description: &AssetDescription,
        source_file: &AssetSourceFile,
    ) -> Self::Cooked;
    fn load(cooked: &Self::Cooked, resources: &mut Resources) -> Self;
}

pub struct InitialAssetDescription {
    pub kind: String,
    pub subresource: String
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetDescription {
    pub asset_id: u64,
    pub asset_name: String,
    pub asset_type: String,
    pub source_file_path: PathBuf,
    pub source_subresource: String,
}
