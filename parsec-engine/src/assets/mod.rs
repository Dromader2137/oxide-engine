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

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct AssetDescription {
    name: String,
    kind: String,
    source_path: PathBuf,
    source_subresource: String,
}
