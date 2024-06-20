use serde::{Deserialize, Serialize};
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};
use log::{debug, error};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, loaders::{gltf::load_gltf, obj::load_obj}, rendering::VertexData, state::State};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mesh {
    pub name: String,
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    #[serde(skip)]
    pub vertex_buffer: Option<Subbuffer<[VertexData]>>,
    #[serde(skip)]
    pub index_buffer: Option<Subbuffer<[u32]>>
}

impl Mesh {
    pub fn new(name: &str, vertices: Vec<VertexData>, indices: Vec<u32>) -> Mesh {
        if vertices.len() == 0 {
            panic!("Empty vertex list not allowed!");
        }
        if indices.len() == 0 {
            panic!("Empty index list not allowed!");
        }
        if *indices.iter().max().unwrap() as usize >= vertices.len() {
            panic!("Index larger than vertex buffer length!");
        }

        Mesh {
            name: name.to_string(),
            vertices: vertices.clone(),
            indices: indices.clone(),
            vertex_buffer: None,
            index_buffer: None
        }
    }

    pub fn load(&mut self, state: &State) {
        self.vertex_buffer = Some(
            Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.vertices.clone()
            ).unwrap()
        );
        self.index_buffer = Some(
            Buffer::from_iter(
                state.renderer.memeory_allocator.clone(),
                BufferCreateInfo {
                    usage: BufferUsage::STORAGE_BUFFER | BufferUsage::SHADER_DEVICE_ADDRESS,
                    ..Default::default()
                },
                AllocationCreateInfo {
                    memory_type_filter: MemoryTypeFilter::PREFER_DEVICE |
                        MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                    ..Default::default()
                },
                self.indices.clone()
            ).unwrap()
        )
    }
}

pub struct DynamicMesh {
    pub material: String,
    pub mesh: Option<String>,
}

impl DynamicMesh {
    pub fn new(material: String) -> Self {
        Self { mesh: None, material }
    }
}

pub struct MeshBufferLoader {}

impl System for MeshBufferLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for mesh in assets.meshes.iter_mut() {
            mesh.load(state);
        }
    }

    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}

pub fn load_model_meshes(assets: &mut AssetLibrary) {
    let len = assets.models.len();
    for i in 0..len {
        let model_name = assets.models.get(i).unwrap().name.clone();
        let mam = match model_name.split_once('.') {
            Some((name, "obj")) => load_obj(name.to_string(), assets).expect("Failed to load"),
            Some((name, "gltf")) => load_gltf(name.to_string(), assets).expect("Failed to load"),
            Some(_) => {
                error!("Unsupportes format {}", model_name);
                continue;
            },
            None => {
                error!("Invalid format");
                continue;
            }
        };
        debug!("{:?}", mam);
        assets.models.get_mut(i).unwrap().meshes_and_materials = mam;
    }
}
