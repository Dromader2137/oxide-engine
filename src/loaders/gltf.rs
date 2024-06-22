use std::{collections::HashMap, path::Path, usize};

use gltf::accessor::DataType;
use log::debug;

use crate::{asset_library::AssetLibrary, rendering::VertexData, types::{material::{Attachment, Material, MaterialParameters}, mesh::Mesh, quaternion::Quat, texture::Texture, vectors::{Vec2f, Vec3f, Vec4f}}};


pub fn load_gltf(
    model_name: String,
    assets: &mut AssetLibrary
) -> Result<Vec<(String, String)>, ()> {
    let mut meshes_and_materials: Vec<(String, String)> = Vec::new();
    let document = gltf::Gltf::open(format!("assets/meshes/{}.gltf", model_name)).unwrap();
    let buffers = gltf::import_buffers(&document, Some(Path::new("assets/meshes/")), None).unwrap();

    let mut materials = HashMap::new();
    for (id, material) in document.materials().enumerate() {
        let name = format!("{}-{}", model_name, id);
        materials.insert(material.index().unwrap(), name.clone());
        let name = name.replace('\\', "/");

        let color_index = match material.pbr_metallic_roughness().base_color_texture() {
            Some(val) => {
                val.texture().index() as i32
            },
            None => -1
        };
        
        let normal_index = match material.normal_texture() {
            Some(val) => {
                val.texture().index() as i32
            },
            None => -1
        };
        
        let mut use_color = 0;
        let mut use_normal = 0;

        let color_name = if color_index >= 0 {
            let color_uri = match document.images().nth(document.textures().nth(color_index as usize).unwrap().source().index()).unwrap().source() {
                gltf::image::Source::View { .. } => "",
                gltf::image::Source::Uri { uri, .. } => uri
            };
            let color_uri = color_uri.replace('\\', "/");
            let name = format!("{}/{}", model_name, color_uri);
            let color_texture = Texture::new(name.clone());
            assets.textures.push(color_texture);
            use_color = 1;
            name
        } else {
            "default".to_string()
        };
        
        let normal_name = if normal_index >= 0 {
            let normal_uri = match document.images().nth(document.textures().nth(normal_index as usize).unwrap().source().index()).unwrap().source() {
                gltf::image::Source::View { .. } => "",
                gltf::image::Source::Uri { uri, .. } => uri
            };
            let normal_uri = normal_uri.replace('\\', "/");
            let name = format!("{}/{}", model_name, normal_uri);
            let normal_texture = Texture::new(name.clone());
            assets.textures.push(normal_texture);
            use_normal = 1;
            name
        } else {
            "default".to_string()
        };

        let mat = Material::new(
            name, 
            "perspective".to_string(),
            "lit".to_string(),
            vec![Attachment::Texture(color_name), Attachment::Texture(normal_name)],
            Some(
                MaterialParameters {
                    diffuse_color: Vec3f::new([1.0, 1.0, 1.0]),
                    use_diffuse_texture: use_color,
                    use_normal_texture: use_normal
                }
            )
        );

        assets.materials.push(mat);
    }
    
    for (n_id, node) in document.nodes().enumerate() {
        let mesh = match node.mesh() {
            Some(val) => val,
            None => continue
        };

        let position = Vec3f::new(node.transform().decomposed().0);
        let rotation = Quat::new_sl(node.transform().decomposed().1);
        let scale = Vec3f::new(node.transform().decomposed().2);
        
        for (prim_id, prim) in mesh.primitives().enumerate() {
            let material_id = prim.material().index().unwrap();
            let mat = materials.get(&material_id).unwrap();
           
            let (_, postition_attribute) = prim.attributes().find(|(attr, _)| {
                match attr {
                    gltf::Semantic::Positions => true,
                    _ => false
                }
            }).expect("Positions requiered");

            let normal_result = prim.attributes().find(|(attr, _)| {
                match attr {
                    gltf::Semantic::Normals => true,
                    _ => false
                }
            });
            
            let uv_result = prim.attributes().find(|(attr, _)| {
                match attr {
                    gltf::Semantic::TexCoords(0) => true,
                    _ => false
                }
            });
            
            let tangent_result = prim.attributes().find(|(attr, _)| {
                match attr {
                    gltf::Semantic::Tangents => true,
                    _ => false
                }
            });

            let positions = {
                let view = postition_attribute.view().unwrap();
                let buffer = buffers.get(view.buffer().index()).unwrap();
                let start = postition_attribute.offset() + view.offset();
                let stride = 12;
                let end = postition_attribute.count() * stride + start;
                let mut pos = Vec::new();

                let mut i = start;
                while i < end {
                    let x = f32::from_le_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                    let y = f32::from_le_bytes([buffer[i+4], buffer[i+5], buffer[i+6], buffer[i+7]]);
                    let z = f32::from_le_bytes([buffer[i+8], buffer[i+9], buffer[i+10], buffer[i+11]]);
                    pos.push(Vec3f::new([x, y, z]));
                   
                    i += stride;
                }

                pos
            };
            let len = positions.len();
            
            let normals = match normal_result {
                Some((_, attribute)) => {
                    let view = attribute.view().unwrap();
                    let buffer = buffers.get(view.buffer().index()).unwrap();
                    let start = attribute.offset() + view.offset();
                    let stride = 12;
                    let end = attribute.count() * stride + start;
                    let mut vec = Vec::new();

                    let mut i = start;
                    while i < end {
                        let x = f32::from_le_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                        let y = f32::from_le_bytes([buffer[i+4], buffer[i+5], buffer[i+6], buffer[i+7]]);
                        let z = f32::from_le_bytes([buffer[i+8], buffer[i+9], buffer[i+10], buffer[i+11]]);
                        vec.push(Vec3f::new([x, y, z]));

                        i += stride;
                    }

                    vec
                },
                None => vec![Vec3f::new([0.0, 1.0, 0.0]); len]
            };
            
            let uvs = match uv_result {
                Some((_, attribute)) => {
                    let view = attribute.view().unwrap();
                    let buffer = buffers.get(view.buffer().index()).unwrap();
                    let start = attribute.offset() + view.offset();
                    let stride = 8;
                    let end = attribute.count() * stride + start;
                    let mut vec = Vec::new();

                    let mut i = start;
                    while i < end {
                        let x = f32::from_le_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                        let y = f32::from_le_bytes([buffer[i+4], buffer[i+5], buffer[i+6], buffer[i+7]]);
                        vec.push(Vec2f::new([x, y]));

                        i += stride;
                    }

                    vec
                },
                None => vec![Vec2f::new([0.0, 1.0]); len]
            };
            
            let tangent = match tangent_result {
                Some((_, attribute)) => {
                    let view = attribute.view().unwrap();
                    let buffer = buffers.get(view.buffer().index()).unwrap();
                    let start = attribute.offset() + view.offset();
                    let stride = 16;
                    let end = attribute.count() * stride + start;
                    let mut vec = Vec::new();

                    let mut i = start;
                    while i < end {
                        let x = f32::from_le_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                        let y = f32::from_le_bytes([buffer[i+4], buffer[i+5], buffer[i+6], buffer[i+7]]);
                        let z = f32::from_le_bytes([buffer[i+8], buffer[i+9], buffer[i+10], buffer[i+11]]);
                        let w = f32::from_le_bytes([buffer[i+12], buffer[i+13], buffer[i+14], buffer[i+15]]);
                        vec.push(Vec4f::new([x, y, z, w]));

                        i += stride;
                    }

                    vec
                },
                None => vec![Vec4f::new([0.0, 1.0, 0.0, 1.0]); len]
            };

            let vertices: Vec<VertexData> = (0..len).map(|i| {
                let tang = *tangent.get(i).unwrap_or(&Vec4f::new([0.0, 1.0, 0.0, 1.0]));
                let tang3 = Vec3f::new([tang.x, tang.y, tang.z]) * rotation;
                let tang = Vec4f::new([tang3.x, tang3.y, tang3.z, tang.w]);
                VertexData {
                    position: *positions.get(i).unwrap() * scale * rotation + position,
                    normal: *normals.get(i).unwrap_or(&Vec3f::new([0.0, 1.0, 0.0])) * rotation,
                    uv: *uvs.get(i).unwrap_or(&Vec2f::new([0.0, 0.0])),
                    tangent: tang 
                }
            }).collect();

            let indices = match prim.indices() { 
                Some(val) => {
                    let view = val.view().unwrap();
                    let buffer = buffers.get(view.buffer().index()).unwrap();
                    let start = val.offset() + view.offset();
                    let count = val.count();
                    let mut vec = Vec::new();

                    match val.data_type() {
                        DataType::U8 => {
                        for i in 0..count {
                            let i = i + start;
                            let x = u32::from_le_bytes([buffer[i], 0x00, 0x00, 0x00]);
                            vec.push(x);
                        }
                        },
                        DataType::U16 => {
                        for i in 0..count {
                            let i = 2 * i + start;
                            let x = u32::from_le_bytes([buffer[i], buffer[i+1], 0x00, 0x00]);
                            vec.push(x);
                        }
                        },
                        DataType::U32 => {
                        for i in 0..count {
                            let i = 4 * i + start;
                            let x = u32::from_le_bytes([buffer[i], buffer[i+1], buffer[i+2], buffer[i+3]]);
                            vec.push(x);
                        }
                        },
                        _ => {
                            log::error!("Type not supported for indices");
                            continue;
                        }
                    }

                    vec
                },
                None => (0..len as u32).collect()
            };

            let name = format!("{}-{}-{}", model_name, n_id, prim_id);
            debug!("Loading mesh {} with material {}...", name, mat);

            assets.meshes.push(
                Mesh::new(&name, vertices, indices)
            );

            meshes_and_materials.push(
                (name,
                mat.to_string()) 
            );
        }
    }


    Ok(meshes_and_materials)
}
