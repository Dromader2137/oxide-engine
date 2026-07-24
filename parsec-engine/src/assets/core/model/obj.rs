use std::collections::HashMap;

use parsec_engine_math::{uvec::{Vec2u, Vec3u}, vec::{Vec2f, Vec3f}};
use thiserror::Error;

use crate::assets::core::{mesh::CookedMesh, model::CookedModel};

#[derive(Error, Debug)]
pub enum LoadOBJError {
    #[error("Failed to load OBJ, because of an IO error: {0}")]
    IOError(#[from] std::io::Error),
    #[error("Failed to load OBJ, beacuse of a parsing error: {0}")]
    FloatParseError(#[from] std::num::ParseFloatError),
    #[error("Failed to load OBJ, beacuse of a parsing error: {0}")]
    IntParseError(#[from] std::num::ParseIntError),
    #[error("Failed to load OBJ, because not all fields are present")]
    NotAllFieldsPresent,
    #[error("Failed to load OBJ, because an undefined position is used")]
    PositionNotFound,
    #[error("Failed to load OBJ, because an undefined uv is used")]
    UvNotFound,
    #[error("Failed to load OBJ, because an undefined normal is used")]
    NormalNotFound,
    #[error("Failed to load OBJ, position coordinate not specified")]
    PositionNotSpecified,
    #[error("Failed to load OBJ, uv coordinate not specified")]
    UVNotSpecified,
    #[error("Failed to load OBJ, normal coordinate not specified")]
    NormalNotSpecified,
    #[error("Failed to load OBJ, index part not specified")]
    IndexNotSpecified,
}

pub fn cook_obj(data: &[u8]) -> Result<CookedModel, LoadOBJError> {
    let mut positions: Vec<Vec3f> = Vec::new();
    let mut uvs: Vec<Vec2f> = Vec::new();
    let mut normals: Vec<Vec3f> = Vec::new();

    let mut out_positions: Vec<Vec3f> = Vec::new();
    let mut out_uvs: Vec<Vec2f> = Vec::new();
    let mut out_normals: Vec<Vec3f> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut index_map: HashMap<(u32, u32, u32), u32> = HashMap::new();

    let objstr = String::from_utf8_lossy(data);
    for line in objstr.lines() {
        if line.is_empty() || line.chars().next() == Some('#') {
            continue;
        }
        let mut tokens = line.split(' ');
        match tokens.next() {
            Some("v") => {
                let mut pos = [0.0_f32; 3];
                for i in 0..3 {
                    pos[i] = tokens
                        .next()
                        .ok_or(LoadOBJError::PositionNotSpecified)?
                        .parse::<f32>()?;
                }
                positions.push(Vec3f::new(pos[0], pos[1], pos[2]));
            },
            Some("vt") => {
                let mut uv = [0.0_f32; 2];
                for i in 0..2 {
                    uv[i] = tokens
                        .next()
                        .ok_or(LoadOBJError::UVNotSpecified)?
                        .parse::<f32>()?;
                }
                uvs.push(Vec2f::new(uv[0], uv[1]));
            },
            Some("vn") => {
                let mut norm = [0.0_f32; 3];
                for i in 0..3 {
                    norm[i] = tokens
                        .next()
                        .ok_or(LoadOBJError::NormalNotSpecified)?
                        .parse::<f32>()?;
                }
                normals.push(Vec3f::new(norm[0], norm[1], norm[2]));
            },
            Some("f") => {
                fn handle_triplet(
                    idx: &mut (u32, u32, u32),
                    mut idxstr: std::str::Split<'_, char>,
                ) -> Result<(), LoadOBJError> {
                    idx.0 = idxstr
                        .next()
                        .ok_or(LoadOBJError::IndexNotSpecified)?
                        .parse::<u32>()?
                        - 1;
                    idx.1 = idxstr
                        .next()
                        .ok_or(LoadOBJError::IndexNotSpecified)?
                        .parse::<u32>()?
                        - 1;
                    idx.2 = idxstr
                        .next()
                        .ok_or(LoadOBJError::IndexNotSpecified)?
                        .parse::<u32>()?
                        - 1;
                    Ok(())
                }

                for _ in 0..3 {
                    let mut idx = (0, 0, 0);
                    let idxstr = tokens
                        .next()
                        .ok_or(LoadOBJError::IndexNotSpecified)?
                        .split('/');
                    handle_triplet(&mut idx, idxstr)?;
                    match index_map.get(&idx) {
                        Some(outidx) => indices.push(*outidx),
                        None => {
                            let realidx = out_positions.len() as u32;
                            index_map.insert(idx, realidx);
                            indices.push(realidx);
                            out_positions.push(positions[idx.0 as usize]);
                            out_uvs.push(uvs[idx.1 as usize]);
                            out_normals.push(normals[idx.2 as usize]);
                        },
                    }
                }
            },
            _ => (),
        }
    }

    Ok(CookedMesh::new(
        out_positions,
        out_normals,
        out_uvs,
        indices,
    ))
}
