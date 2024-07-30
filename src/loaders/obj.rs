use std::collections::{HashMap, HashSet};

use log::{debug, error};
use uuid::Uuid;

use crate::{asset_library::AssetLibrary, rendering::VertexData, types::{material::{Attachment, Material, MaterialParameters, RenderingType}, mesh::Mesh, texture::Texture, vectors::{Vec2f, Vec3f, Vec4f}}};

#[allow(clippy::result_unit_err)]
pub fn load_obj(
    model_name: String,
    assets: &mut AssetLibrary
) -> Result<Vec<(Uuid, Uuid)>, ()> {
    let mut meshes_and_materials = Vec::new();
        let obj = tobj::load_obj(format!("assets/meshes/{}.obj", model_name), &tobj::GPU_LOAD_OPTIONS);
        let (meshes, materials) = match obj {
            Ok(val) => val,
            Err(_) => {
                error!("Failed to load {}", model_name);
                return Err(());
            }
        };

        let materials = match materials {
            Ok(val) => val,
            Err(e) => {
                error!("Material loading error for {}: {}", model_name, e);
                return Err(());
            }
        };

        let mut material_map = HashMap::new();
        for material in materials.iter() {
            let uuid = Uuid::new_v4();
            let name = format!("{}-{}", model_name, material.name);
            material_map.insert(name.clone(), uuid);

            assets.materials.insert(
                uuid,
                Material::new(
                    name.clone(),
                    *assets.shaders.iter().find(|(_, v)| v.name.as_str() == "perspective").expect("\"perspective\" shader needed").0,
                    *assets.shaders.iter().find(|(_, v)| v.name.as_str() == "lit").expect("\"lit\" shader needed").0,
                    vec![
            {
                match &material.diffuse_texture {
                    Some(val) => {
                        let name = format!("{}/{}", model_name, val).replace('\\', "/");
                        let uuid = Uuid::new_v4();
                        assets.textures.insert(uuid, Texture::new(name));
                        Attachment::Texture(uuid)
                    },
                    None => Attachment::DefaultTexture
                }
            },
            {
                match &material.normal_texture {
                    Some(val) => {
                        let name = format!("{}/{}", model_name, val).replace('\\', "/");
                        let uuid = Uuid::new_v4();
                        assets.textures.insert(uuid, Texture::new(name));
                        Attachment::Texture(uuid)
                    },
                    None => Attachment::DefaultTexture
                }
            },
                    ],
                    Some(MaterialParameters {
                        diffuse_color: match material.diffuse {
                            Some(col) => Vec3f::new(col),
                            None => Vec3f::new([1.0, 0.0, 1.0])
                        },
                        use_normal_texture: match &material.normal_texture {
                            Some(_) => 1,
                            None => 0
                        },
                        use_diffuse_texture: match &material.diffuse_texture {
                            Some(_) => 1,
                            None => 0
                        }
                    }),
            RenderingType::Fill
            )
                );
        }

        let mut used_names: HashSet<String> = HashSet::new();
        for (id, mesh) in meshes.iter().enumerate() {
            let mat = match mesh.mesh.material_id {
                Some(material_id) => {
                    match materials.get(material_id) {
                        Some(material) => {
                            material_map.get(&format!("{}-{}", model_name, material.name)).unwrap()
                        },
                        None => {
                            error!("Material loading error for {}: material id not found", model_name);
                            return Err(());
                        }

                    }
                },
                None => {
                    error!("Material loading error for {}: objects without material currently not supported", model_name);
                    return Err(());
                }
            };
            let name = format!("{}{}", model_name, id);
            let ret = used_names.insert(name.clone());
            if !ret {
                error!("Name repetition!");
                return Err(());
            }
            debug!("Loading mesh {} with material {}...", name, mat);

            let pos = &mesh.mesh.positions;
            let nor = &mesh.mesh.normals;
            let uvs = &mesh.mesh.texcoords;

            let mut vertices: Vec<VertexData> = Vec::new(); 
            for i in 0..pos.len()/3 {
                let normal = if nor.get(3*i+2).is_some() {
                    Vec3f::new([nor[3*i],nor[3*i+1],nor[3*i+2]])
                } else {
                    Vec3f::new([0.0, 1.0, 0.0])
                };

                let uv = if uvs.get(2*i+1).is_some() {
                    Vec2f::new([uvs[2*i], -uvs[2*i+1]])
                } else {
                    Vec2f::new([0.0, 1.0])
                };

                vertices.push(
                    VertexData {
                        position: Vec3f::new([pos[3*i],pos[3*i+1],pos[3*i+2]]),
                        normal,
                        uv,
                        tangent: Vec4f::new([0.0, 1.0, 0.0, 1.0])
                }
                );
            }

            let uuid = Uuid::new_v4();
            assets.meshes.insert(uuid, Mesh::new(&name, vertices, mesh.mesh.indices.clone()));

            meshes_and_materials.push(
                (
                    uuid,
                    *mat
                )
            );
        }

        Ok(meshes_and_materials)
}
