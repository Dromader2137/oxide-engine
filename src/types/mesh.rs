use std::collections::HashSet;

use log::{debug, error};
use vulkano::{buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer}, memory::allocator::{AllocationCreateInfo, MemoryTypeFilter}};

use crate::{asset_library::AssetLibrary, ecs::{System, World}, rendering::VertexData, state::State, types::vectors::{Vec2f, Vec3f}, types::texture::Texture};

use super::material::{Attachment, Material, MaterialParameters};

#[derive(Debug, Clone)]
pub struct Mesh {
    pub vertices: Vec<VertexData>,
    pub indices: Vec<u32>,
    pub vertex_buffer: Subbuffer<[VertexData]>,
    pub index_buffer: Subbuffer<[u32]>
}

impl Mesh {
    pub fn new(vertices: Vec<VertexData>, indices: Vec<u32>, state: &State) -> Mesh {
        if vertices.len() == 0 {
            panic!("Empty vertex list not allowed!");
        }
        if indices.len() == 0 {
            panic!("Empty index list not allowed!");
        }

        Mesh {
            vertices: vertices.clone(),
            indices: indices.clone(),
            vertex_buffer: Buffer::from_iter(
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
                vertices
            ).unwrap(),
            index_buffer: Buffer::from_iter(
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
                indices
            ).unwrap()
        }
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

pub struct MeshLoader {}

impl System for MeshLoader {
    fn on_start(&self, _world: &World, assets: &mut AssetLibrary, state: &mut State) {
        for model in assets.models.iter_mut() {
            let mesh_name = &model.name;
            let obj = tobj::load_obj(format!("assets/meshes/{}.obj", mesh_name), &tobj::GPU_LOAD_OPTIONS);
            let (meshes, materials) = match obj {
                Ok(val) => val,
                Err(_) => {
                    error!("Failed to load {}", mesh_name);
                    continue;
                }
            };

            let materials = match materials {
                Ok(val) => val,
                Err(e) => {
                    error!("Material loading error for {}: {}", mesh_name, e);
                    continue;
                }
            };

            for material in materials.iter() {
                let name = format!("{}{}", mesh_name, material.name);
                if assets.materials.iter().find(|x| x.name == name).is_some() { continue; }

                assets.materials.push(
                    Material::new(
                        &name,
                        "perspective",
                        "lit",
                        vec![
                            {
                                match &material.diffuse_texture {
                                    Some(val) => {
                                        let name = format!("{}/{}", mesh_name, val);
                                        let name = name.replace('\\', "/");
                                        debug!("{}", name);
                                        assets.textures.push(Texture::new(name.clone()));
                                        Attachment::Texture(name.clone())
                                    }
                                    None => Attachment::Texture("default".to_string())
                                }
                            }
                        ],
                        Some(MaterialParameters {
                            diffuse_color: match material.diffuse {
                                Some(col) => Vec3f::new(col),
                                None => Vec3f::new([0.0, 0.0, 0.0])
                            },
                            roughness: match material.shininess {
                                Some(val) => 1.0 - val,
                                None => 1.0
                            }
                        })
                    )
                );
            }

            let mut used_names: HashSet<String> = HashSet::new();
            for (id, mesh) in meshes.iter().enumerate() {
                let mat = match mesh.mesh.material_id {
                    Some(material_id) => {
                        match materials.get(material_id) {
                            Some(material) => format!("{}{}", mesh_name, material.name),
                            None => {
                                error!("Material loading error for {}: material id not found", mesh_name);
                                continue;
                            }

                        }
                    },
                    None => "default".to_string()
                };
                let name = format!("{}{}", mesh_name, id);
                let ret = used_names.insert(name.clone());
                if !ret {
                    error!("Name repetition!");
                    continue;
                }
                debug!("Loading {} {}...", name, mat);

                let pos = &mesh.mesh.positions;
                let nor = &mesh.mesh.normals;
                let uvs = &mesh.mesh.texcoords;
                debug!("{} {} {}", pos.len(), nor.len(), uvs.len());

                let mut vertices: Vec<VertexData> = Vec::new(); 
                for i in 0..pos.len()/3 {
                    vertices.push(
                        VertexData {
                            position: Vec3f::new([pos[3*i],pos[3*i+1],pos[3*i+2]]),
                            normal: Vec3f::new([nor[3*i],nor[3*i+1],nor[3*i+2]]),
                            uv: Vec2f::new([uvs[2*i], uvs[2*i+1]])
                        }
                    );
                }

                state.meshes.insert(
                    name.clone(), 
                    Mesh::new(vertices, mesh.mesh.indices.clone(), state),
                );
                
                model.meshes_and_materials.push((
                    name,
                    mat
                ));
            }
        }
    }
    fn on_update(&self, _world: &World, _assets: &mut AssetLibrary, _state: &mut State) {}
}
