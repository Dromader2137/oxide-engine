use crate::{
    assets::{
        Asset, AssetDescription,
        core::{
            mesh::{CookedMesh, Mesh},
            model::obj::cook_obj,
        },
    },
    ecs::resources::Resources,
};

mod obj;

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CookedModel {
    meshes: Vec<CookedMesh>,
}

#[derive(Debug)]
pub struct Model {
    meshes: Vec<Mesh>,
}

impl Asset for Model {
    type Cooked = CookedModel;

    const ASSET_TYPE: &'static str = "model";
    const EXTENSIONS: &'static [&'static str] = &["obj"];

    fn cook(data: &[u8], asset_description: &AssetDescription) -> Self::Cooked {
        let extension = asset_description
            .path
            .extension()
            .expect("models has to have an extension");
        if extension == "obj" {
            let cooked = cook_obj(data).unwrap();
            return cooked;
        }
        CookedModel::default()
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self {
        let len = cooked.meshes.len();
        let Self::Cooked { mut meshes } = cooked;
        let meshes = meshes
            .drain(0..len)
            .map(|x| Mesh::load(x, resources))
            .collect();
        Model { meshes }
    }
}
