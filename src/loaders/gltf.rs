use std::path::Path;

use log::debug;

use crate::{asset_library::AssetLibrary, rendering::VertexData, types::texture::Texture};


pub fn load_gltf(
    model_name: String,
    assets: &mut AssetLibrary
) -> Result<Vec<(String, String)>, ()> {
    let mut meshes_and_materials: Vec<(String, String)> = Vec::new();
    let document = gltf::Gltf::open(format!("assets/meshes/{}.gltf", model_name)).unwrap();
    let buffers = gltf::import_buffers(&document, Some(Path::new("assets/meshes/")), None).unwrap();
    // let images = gltf::import_images(&document, Some(Path::new(&format!("assets/textures/{}/", model_name))), buffers.as_slice()).unwrap();

    for material in document.materials() {
        let name = format!("{}/{}", model_name, material.name().unwrap());
        let name = name.replace('\\', "/");

        let color_index = match material.pbr_metallic_roughness().base_color_texture() {
            Some(val) => {
                val.tex_coord() as i32
            },
            None => -1
        };
        
        let normal_index = match material.normal_texture() {
            Some(val) => {
                val.tex_coord() as i32
            },
            None => -1
        };
        
        debug!("{} {} {}", color_index, normal_index, name);

        let color_name = if color_index >= 0 {
            let color_uri = match document.images().nth(color_index as usize).unwrap().source() {
                gltf::image::Source::View { .. } => "",
                gltf::image::Source::Uri { uri, .. } => uri
            };
            let color_uri = color_uri.replace('\\', "/");
            let name = format!("{}/{}", model_name, color_uri);
            let color_texture = Texture::new(name.clone());
            assets.textures.push(color_texture);
            name
        } else {
            "default".to_string()
        };
        
        let normal_name = if normal_index >= 0 {
            let normal_uri = match document.images().nth(normal_index as usize).unwrap().source() {
                gltf::image::Source::View { .. } => "",
                gltf::image::Source::Uri { uri, .. } => uri
            };
            let normal_uri = normal_uri.replace('\\', "/");
            let name = format!("{}/{}", model_name, normal_uri);
            let normal_texture = Texture::new(name.clone());
            assets.textures.push(normal_texture);
            name
        } else {
            "default".to_string()
        };
    }

    for (m_id, mesh) in document.meshes().enumerate() {
        for (prim_id, prim) in mesh.primitives().enumerate() {
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

            let vertices: Vec<VertexData> = Vec::new();
            for i in postition_attribute.view().unwrap(). {

            }
        }
    }


    Ok(meshes_and_materials)
}
