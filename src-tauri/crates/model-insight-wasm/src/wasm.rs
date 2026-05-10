use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::{MdfFile, MeshFile};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MeshPreviewResponse {
    mesh_file_version: u32,
    mdf_file_version: Option<u32>,
    preview: crate::PreviewModel,
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = initModelInsightWasm)]
pub fn init_model_insight_wasm() {
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = modelInsightWasmVersion)]
pub fn model_insight_wasm_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[wasm_bindgen(js_name = meshToPreviewModel)]
pub fn mesh_to_preview_model(
    mesh_bytes: &[u8],
    mesh_file_version: u32,
    mdf_bytes: Option<Vec<u8>>,
    mdf_file_version: Option<u32>,
) -> Result<JsValue, JsValue> {
    let mesh = MeshFile::read_bytes(mesh_bytes, mesh_file_version).map_err(js_error)?;
    let mut preview = mesh.to_preview_model().map_err(js_error)?;

    if let (Some(mdf_bytes), Some(mdf_file_version)) = (mdf_bytes, mdf_file_version) {
        let mdf = MdfFile::read_bytes(&mdf_bytes, mdf_file_version).map_err(js_error)?;
        preview.materials = mdf.preview_materials_for(&mesh.material_names);
    }

    serde_wasm_bindgen::to_value(&MeshPreviewResponse {
        mesh_file_version,
        mdf_file_version,
        preview,
    })
    .map_err(js_error)
}

fn js_error(error: impl ToString) -> JsValue {
    JsValue::from_str(&error.to_string())
}
