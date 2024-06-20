use std::collections::HashSet;

use log::{debug, error};

use crate::{asset_library::AssetLibrary, rendering::VertexData, types::{material::{Attachment, Material, MaterialParameters}, mesh::Mesh, texture::Texture, vectors::{Vec2f, Vec3f}}};


pub fn load_obj(
    model_name: String,
    assets: &mut AssetLibrary
) -> Result<Vec<(String, String)>, ()> {
    let mut meshes_and_materials: Vec<(String, String)> = Vec::new();
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

        for material in materials.iter() {
            let name = format!("{}{}", model_name, material.name);
            if assets.materials.iter().find(|x| x.name == name).is_some() { return Err(()); }

            assets.materials.push(
                Material::new(
                    name.clone(),
                    "perspective".to_string(),
                    "lit".to_string(),
                    vec![
            {
                match &material.diffuse_texture {
                    Some(val) => {
                        let name = format!("{}/{}", model_name, val);
                        let name = name.replace('\\', "/");
                        assets.textures.push(Texture::new(name.clone()));
                        Attachment::Texture(name.clone())
                    }
                    None => Attachment::Texture("default".to_string())
                }
            },
            {
                match &material.normal_texture {
                    Some(val) => {
                        let name = format!("{}/{}", model_name, val);
                        let name = name.replace('\\', "/");
                        assets.textures.push(Texture::new(name.clone()));
                        Attachment::Texture(name.clone())
                    }
                    None => Attachment::Texture("default".to_string())
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
                    })
            )
                );
        }

        let mut used_names: HashSet<String> = HashSet::new();
        for (id, mesh) in meshes.iter().enumerate() {
            let mat = match mesh.mesh.material_id {
                Some(material_id) => {
                    match materials.get(material_id) {
                        Some(material) => format!("{}{}", model_name, material.name),
                        None => {
                            error!("Material loading error for {}: material id not found", model_name);
                            return Err(());
                        }

                    }
                },
                None => "default".to_string()
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
                        tangent: Vec3f::new([0.0, 1.0, 0.0])
                }
                );
            }

            assets.meshes.push(
                Mesh::new(&name, vertices, mesh.mesh.indices.clone()),
            );

            meshes_and_materials.push((
                    name,
                    mat
            ));
        }

        Ok(meshes_and_materials)
}
