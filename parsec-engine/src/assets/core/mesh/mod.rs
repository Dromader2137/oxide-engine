use parsec_engine_math::vec::{Vec2f, Vec3f};

use crate::{
    assets::{Asset, AssetDescription},
    create_counter,
    ecs::resources::Resources,
    error::OptionNoneErr,
    graphics::{ActiveGraphicsBackend, pipeline::DefaultVertex},
    renderer::mesh_data::MeshData,
    utils::{
        IdType,
        identifiable::{IdStore, Identifiable},
    },
};

#[derive(Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct CookedMesh {
    positions: Vec<Vec3f>,
    normals: Vec<Vec3f>,
    uvs: Vec<Vec2f>,
    indices: Vec<u32>,
}

impl CookedMesh {
    pub fn new(
        positions: Vec<Vec3f>,
        normals: Vec<Vec3f>,
        uvs: Vec<Vec2f>,
        indices: Vec<u32>,
    ) -> Self {
        Self {
            positions,
            normals,
            uvs,
            indices,
        }
    }
}

#[derive(Debug)]
pub struct Mesh {
    mesh_id: IdType,
    pub vertices: Vec<DefaultVertex>,
    pub indices: Vec<u32>,
    pub data_id: Option<u32>,
}

create_counter! {ID_COUNTER}
impl Mesh {
    pub fn new(vertices: Vec<DefaultVertex>, indices: Vec<u32>) -> Mesh {
        Mesh {
            mesh_id: ID_COUNTER.next(),
            vertices,
            indices,
            data_id: None,
        }
    }
}

impl Identifiable for Mesh {
    fn id(&self) -> IdType { self.mesh_id }
}

impl From<CookedMesh> for Mesh {
    fn from(mut value: CookedMesh) -> Self {
        let pos_len = value.positions.len();
        value.uvs.resize(pos_len, Vec2f::ZERO);
        value.normals.resize(pos_len, Vec3f::UP);
        let vertices = value
            .positions
            .iter()
            .zip(value.uvs.iter())
            .zip(value.normals.iter())
            .map(|((pos, uv), norm)| DefaultVertex::new(*pos, *norm, *uv))
            .collect();
        Self::new(vertices, value.indices)
    }
}

impl Asset for Mesh {
    type Cooked = CookedMesh;

    const ASSET_TYPE: &'static str = "mesh";
    const EXTENSIONS: &'static [&'static str] = &[];

    fn cook(
        _data: &[u8],
        _asset_description: &AssetDescription,
    ) -> Self::Cooked {
        CookedMesh::default()
    }

    fn load(cooked: Self::Cooked, resources: &mut Resources) -> Self {
        let mut mesh = Mesh::from(cooked);
        let mesh_data = {
            let mut backend = resources
                .get_mut::<ActiveGraphicsBackend>()
                .none_err()
                .unwrap();
            MeshData::new(&mut backend, &mesh.vertices, &mesh.indices)
        };
        let mut mesh_data_store = resources
            .get_add::<IdStore<MeshData<DefaultVertex>>>(IdStore::new());
        let id = mesh_data_store.push(mesh_data);
        mesh.data_id = Some(id);
        mesh
    }
}
