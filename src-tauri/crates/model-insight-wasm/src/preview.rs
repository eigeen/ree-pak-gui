use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewModel {
    pub meshes: Vec<PreviewSubmesh>,
    pub materials: Vec<PreviewMaterial>,
    pub bounds: PreviewBounds,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewSubmesh {
    pub name: String,
    pub material_index: usize,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub indices: Vec<u32>,
}

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewMaterial {
    pub name: String,
    pub albedo_texture_path: Option<String>,
    pub normal_texture_path: Option<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PreviewBounds {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub sphere_center: [f32; 3],
    pub sphere_radius: f32,
}
