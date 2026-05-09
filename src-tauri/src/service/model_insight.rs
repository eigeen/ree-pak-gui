use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
    process::Command,
    sync::{Arc, OnceLock},
    thread,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    TEMP_DIR_NAME,
    common::JsSafeHash,
    error::{Error, Result},
    external_tools, get_local_dir,
    pak::{Pak, PakId},
    service::pak::PakService,
};

static MODEL_INSIGHT_SERVICE: OnceLock<ModelInsightService> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightOpenMeshOptions {
    pub hash: JsSafeHash,
    pub belongs_to: Option<PakId>,
    pub entry_path: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightRenderMeshOptions {
    pub hash: JsSafeHash,
    pub belongs_to: Option<PakId>,
    pub entry_path: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub camera_yaw: Option<f32>,
    pub camera_pitch: Option<f32>,
    pub camera_distance_scale: Option<f32>,
    pub frame_y: Option<f32>,
    pub start_resident_viewer: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightLaunchInfo {
    pub session_id: String,
    pub manifest_path: String,
    pub rpc_addr: String,
    pub executable_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ModelInsightRequestManifest {
    session_id: String,
    rpc_addr: String,
    token: String,
    mesh_entry_path: String,
    mesh_hash: JsSafeHash,
}

#[derive(Debug, Clone)]
struct ModelInsightSession {
    token: String,
    temp_dir: PathBuf,
    mesh_hash: u64,
    mesh_belongs_to: Option<PakId>,
    mesh_entry_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AssetMaterializeResponse {
    local_path: String,
    entry_path: String,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonRpcRequest {
    id: u64,
    method: String,
    params: Value,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AssetMaterializeParams {
    session_id: String,
    token: String,
    kind: String,
    path: Option<String>,
    base: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RegisterViewerParams {
    session_id: String,
    token: String,
    control_addr: String,
}

#[derive(Debug, Clone, Serialize)]
struct RegisterViewerResponse {
    registered: bool,
}

#[derive(Debug, Clone, Serialize)]
struct ControlLoadMeshRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: ControlLoadMeshParams,
}

#[derive(Debug, Clone, Serialize)]
struct ControlRenderMeshPreviewRequest {
    jsonrpc: &'static str,
    id: u64,
    method: &'static str,
    params: ControlRenderMeshPreviewParams,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ControlLoadMeshParams {
    manifest_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ControlRenderMeshPreviewParams {
    manifest_path: String,
    output_path: String,
    width: u32,
    height: u32,
    camera_yaw: f32,
    camera_pitch: f32,
    camera_distance_scale: f32,
    frame_y: f32,
}

#[derive(Debug, Clone, Deserialize)]
struct ControlLoadMeshResponse {
    result: Option<ControlOk>,
    error: Option<JsonRpcErrorMessage>,
}

#[derive(Debug, Clone, Deserialize)]
struct ControlOk {
    ok: bool,
}

#[derive(Debug, Clone, Deserialize)]
struct JsonRpcErrorMessage {
    code: i32,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct JsonRpcResponse<T> {
    jsonrpc: &'static str,
    id: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Clone, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
}

#[derive(Clone)]
struct BrokerState {
    pak_service: &'static PakService,
    sessions: Arc<parking_lot::Mutex<HashMap<String, ModelInsightSession>>>,
    viewer: Arc<parking_lot::Mutex<Option<ModelInsightInstance>>>,
}

struct BrokerRuntime {
    port: u16,
}

#[derive(Debug, Clone)]
struct ModelInsightInstance {
    control_addr: String,
}

pub struct ModelInsightService {
    pak_service: &'static PakService,
    temp_dir: PathBuf,
    sessions: Arc<parking_lot::Mutex<HashMap<String, ModelInsightSession>>>,
    viewer: Arc<parking_lot::Mutex<Option<ModelInsightInstance>>>,
    broker: parking_lot::Mutex<Option<BrokerRuntime>>,
    resident_viewer_start: parking_lot::Mutex<Option<Instant>>,
}

const DEFAULT_RENDER_PREVIEW_SIZE: u32 = 256;
const MODEL_PREVIEW_CACHE_VERSION: &str = "v3";
const DEFAULT_RENDER_CAMERA_YAW: f32 = -0.65;
const DEFAULT_RENDER_CAMERA_PITCH: f32 = -0.28;
const DEFAULT_RENDER_CAMERA_DISTANCE_SCALE: f32 = 2.6;
const DEFAULT_RENDER_FRAME_Y: f32 = 0.0;

#[derive(Debug, Clone, Copy)]
struct RenderMeshCameraOptions {
    yaw: f32,
    pitch: f32,
    distance_scale: f32,
    frame_y: f32,
}

fn render_mesh_camera_options(
    options: &ModelInsightRenderMeshOptions,
) -> Result<RenderMeshCameraOptions> {
    let camera = RenderMeshCameraOptions {
        yaw: options.camera_yaw.unwrap_or(DEFAULT_RENDER_CAMERA_YAW),
        pitch: options.camera_pitch.unwrap_or(DEFAULT_RENDER_CAMERA_PITCH),
        distance_scale: options
            .camera_distance_scale
            .unwrap_or(DEFAULT_RENDER_CAMERA_DISTANCE_SCALE),
        frame_y: options.frame_y.unwrap_or(DEFAULT_RENDER_FRAME_Y),
    };

    if !camera.yaw.is_finite()
        || !camera.pitch.is_finite()
        || !camera.distance_scale.is_finite()
        || !camera.frame_y.is_finite()
    {
        return Err(Error::Internal(
            "render camera options must be finite numbers".to_string(),
        ));
    }
    if camera.distance_scale <= 0.0 {
        return Err(Error::Internal(
            "render camera distance scale must be greater than 0".to_string(),
        ));
    }

    Ok(camera)
}

fn float_cache_key(value: f32) -> String {
    format!("{value:.3}").replace('-', "m").replace('.', "p")
}

impl ModelInsightService {
    pub fn initialize() -> Result<&'static Self> {
        let temp_dir = get_local_dir().join(TEMP_DIR_NAME).join("model-insight");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(MODEL_INSIGHT_SERVICE.get_or_init(|| Self {
            pak_service: PakService::get(),
            temp_dir,
            sessions: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            viewer: Arc::new(parking_lot::Mutex::new(None)),
            broker: parking_lot::Mutex::new(None),
            resident_viewer_start: parking_lot::Mutex::new(None),
        }))
    }

    pub fn get() -> &'static Self {
        MODEL_INSIGHT_SERVICE.get().unwrap()
    }

    pub fn open_mesh(
        &self,
        options: ModelInsightOpenMeshOptions,
    ) -> Result<ModelInsightLaunchInfo> {
        let request = self.create_request(options.hash, options.belongs_to, &options.entry_path)?;

        if self
            .send_load_to_existing_viewer(&request.manifest_path)
            .is_ok()
        {
            return Ok(ModelInsightLaunchInfo {
                session_id: request.session_id,
                manifest_path: request.manifest_path.to_string_lossy().to_string(),
                rpc_addr: request.rpc_addr,
                executable_path: self.viewer_executable_hint().unwrap_or_default(),
            });
        }

        let executable_path = external_tools::find_model_insight_cli().ok_or_else(|| {
            Error::Internal(format!(
                "model-insight not found. Place it under: {}",
                external_tools::model_insight_status().expected_path
            ))
        })?;

        Command::new(&executable_path)
            .args(["view", "--request"])
            .arg(&request.manifest_path)
            .spawn()
            .map_err(|error| Error::Internal(format!("failed to start model-insight: {error}")))?;

        Ok(ModelInsightLaunchInfo {
            session_id: request.session_id,
            manifest_path: request.manifest_path.to_string_lossy().to_string(),
            rpc_addr: request.rpc_addr,
            executable_path: executable_path.to_string_lossy().to_string(),
        })
    }

    pub fn render_mesh_preview(&self, options: ModelInsightRenderMeshOptions) -> Result<String> {
        let width = options.width.unwrap_or(DEFAULT_RENDER_PREVIEW_SIZE);
        let height = options.height.unwrap_or(DEFAULT_RENDER_PREVIEW_SIZE);
        let camera = render_mesh_camera_options(&options)?;
        if width == 0 || height == 0 {
            return Err(Error::Internal(
                "render width and height must be greater than 0".to_string(),
            ));
        }

        let output_path = self.preview_output_path(
            options.hash.hash_u64(),
            options.belongs_to.as_ref(),
            &options.entry_path,
            width,
            height,
            camera,
        );
        if output_path.exists() {
            return Ok(output_path.to_string_lossy().to_string());
        }

        if let Some(parent) = output_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let request = self.create_request(options.hash, options.belongs_to, &options.entry_path)?;
        if self
            .send_render_to_existing_viewer(
                &request.manifest_path,
                &output_path,
                width,
                height,
                camera,
            )
            .is_ok()
        {
            return Ok(output_path.to_string_lossy().to_string());
        }

        if options.start_resident_viewer.unwrap_or(false)
            && self
                .render_with_resident_viewer(
                    &request.manifest_path,
                    &output_path,
                    width,
                    height,
                    camera,
                )
                .is_ok()
        {
            return Ok(output_path.to_string_lossy().to_string());
        }

        let executable_path = external_tools::find_model_insight_cli().ok_or_else(|| {
            Error::Internal(format!(
                "model-insight not found. Place it under: {}",
                external_tools::model_insight_status().expected_path
            ))
        })?;
        let width_arg = width.to_string();
        let height_arg = height.to_string();
        let camera_yaw_arg = camera.yaw.to_string();
        let camera_pitch_arg = camera.pitch.to_string();
        let camera_distance_scale_arg = camera.distance_scale.to_string();
        let frame_y_arg = camera.frame_y.to_string();

        let output = Command::new(&executable_path)
            .args([
                "render",
                "--width",
                width_arg.as_str(),
                "--height",
                height_arg.as_str(),
                "--camera-yaw",
                camera_yaw_arg.as_str(),
                "--camera-pitch",
                camera_pitch_arg.as_str(),
                "--camera-distance-scale",
                camera_distance_scale_arg.as_str(),
                "--frame-y",
                frame_y_arg.as_str(),
                "--request",
            ])
            .arg(&request.manifest_path)
            .arg(&output_path)
            .output()
            .map_err(|error| Error::Internal(format!("failed to start model-insight: {error}")))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            let detail = if stderr.trim().is_empty() {
                stdout.trim()
            } else {
                stderr.trim()
            };
            return Err(Error::Internal(format!(
                "model-insight render failed: {detail}"
            )));
        }

        Ok(output_path.to_string_lossy().to_string())
    }

    fn create_request(
        &self,
        hash: JsSafeHash,
        belongs_to: Option<PakId>,
        entry_path: &str,
    ) -> Result<ModelInsightRequest> {
        let port = self.ensure_broker()?;
        let session_id = Uuid::new_v4().to_string();
        let token = Uuid::new_v4().to_string();
        let session_dir = self.temp_dir.join(&session_id);
        std::fs::create_dir_all(&session_dir)?;
        let mesh_entry_path = normalize_entry_path(entry_path);

        let session = ModelInsightSession {
            token: token.clone(),
            temp_dir: session_dir,
            mesh_hash: hash.hash_u64(),
            mesh_belongs_to: belongs_to,
            mesh_entry_path: mesh_entry_path.clone(),
        };
        self.sessions.lock().insert(session_id.clone(), session);

        let rpc_addr = format!("127.0.0.1:{port}");
        let manifest = ModelInsightRequestManifest {
            session_id: session_id.clone(),
            rpc_addr: rpc_addr.clone(),
            token,
            mesh_entry_path,
            mesh_hash: hash,
        };
        let manifest_path = self.temp_dir.join(format!("{session_id}.json"));
        let manifest_file = File::create(&manifest_path)?;
        serde_json::to_writer_pretty(manifest_file, &manifest)
            .map_err(|error| Error::Internal(error.to_string()))?;

        Ok(ModelInsightRequest {
            session_id,
            manifest_path,
            rpc_addr,
        })
    }

    fn preview_output_path(
        &self,
        hash: u64,
        belongs_to: Option<&PakId>,
        entry_path: &str,
        width: u32,
        height: u32,
        camera: RenderMeshCameraOptions,
    ) -> PathBuf {
        let pak_key = belongs_to
            .map(|value| sanitize_file_name(&format!("{value:?}")))
            .unwrap_or_else(|| "unknown".to_string());
        let file_key = sanitize_file_name(file_name(entry_path));
        let camera_key = format!(
            "yaw{}-pitch{}-dist{}-frame{}",
            float_cache_key(camera.yaw),
            float_cache_key(camera.pitch),
            float_cache_key(camera.distance_scale),
            float_cache_key(camera.frame_y)
        );
        self.temp_dir.join("previews").join(format!(
            "{MODEL_PREVIEW_CACHE_VERSION}-{hash:016X}-{pak_key}-{width}x{height}-{camera_key}-{file_key}.png"
        ))
    }

    fn ensure_broker(&self) -> Result<u16> {
        if let Some(runtime) = self.broker.lock().as_ref() {
            return Ok(runtime.port);
        }

        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        let state = BrokerState {
            pak_service: self.pak_service,
            sessions: self.sessions.clone(),
            viewer: self.viewer.clone(),
        };
        thread::Builder::new()
            .name("model-insight-asset-broker".to_string())
            .spawn(move || run_broker(listener, state))
            .map_err(|error| Error::Internal(error.to_string()))?;

        *self.broker.lock() = Some(BrokerRuntime { port });
        Ok(port)
    }

    fn send_load_to_existing_viewer(&self, manifest_path: &PathBuf) -> Result<()> {
        let Some(instance) = self.viewer.lock().clone() else {
            return Err(Error::Internal(
                "model-insight viewer is not registered".to_string(),
            ));
        };

        let request = ControlLoadMeshRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "loadMesh",
            params: ControlLoadMeshParams {
                manifest_path: manifest_path.to_string_lossy().to_string(),
            },
        };
        let mut stream = TcpStream::connect(&instance.control_addr).map_err(|error| {
            *self.viewer.lock() = None;
            Error::Internal(format!("registered model-insight is offline: {error}"))
        })?;
        write_json_packet(&mut stream, &request)?;
        let response: ControlLoadMeshResponse = read_json_packet(&mut stream)?;
        if let Some(error) = response.error {
            *self.viewer.lock() = None;
            return Err(Error::Internal(format!(
                "model-insight load failed: {} {}",
                error.code, error.message
            )));
        }
        if response.result.is_some_and(|result| result.ok) {
            return Ok(());
        }
        Err(Error::Internal(
            "model-insight load response missing ok result".to_string(),
        ))
    }

    fn send_render_to_existing_viewer(
        &self,
        manifest_path: &PathBuf,
        output_path: &PathBuf,
        width: u32,
        height: u32,
        camera: RenderMeshCameraOptions,
    ) -> Result<()> {
        let Some(instance) = self.viewer.lock().clone() else {
            return Err(Error::Internal(
                "model-insight viewer is not registered".to_string(),
            ));
        };

        let request = ControlRenderMeshPreviewRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "renderMeshPreview",
            params: ControlRenderMeshPreviewParams {
                manifest_path: manifest_path.to_string_lossy().to_string(),
                output_path: output_path.to_string_lossy().to_string(),
                width,
                height,
                camera_yaw: camera.yaw,
                camera_pitch: camera.pitch,
                camera_distance_scale: camera.distance_scale,
                frame_y: camera.frame_y,
            },
        };
        let mut stream = TcpStream::connect(&instance.control_addr).map_err(|error| {
            *self.viewer.lock() = None;
            Error::Internal(format!("registered model-insight is offline: {error}"))
        })?;
        write_json_packet(&mut stream, &request)?;
        let response: ControlLoadMeshResponse = read_json_packet(&mut stream).map_err(|error| {
            *self.viewer.lock() = None;
            error
        })?;
        if let Some(error) = response.error {
            return Err(Error::Internal(format!(
                "model-insight render failed: {} {}",
                error.code, error.message
            )));
        }
        if response.result.is_some_and(|result| result.ok) {
            return Ok(());
        }
        Err(Error::Internal(
            "model-insight render response missing ok result".to_string(),
        ))
    }

    fn render_with_resident_viewer(
        &self,
        manifest_path: &PathBuf,
        output_path: &PathBuf,
        width: u32,
        height: u32,
        camera: RenderMeshCameraOptions,
    ) -> Result<()> {
        self.launch_resident_viewer(manifest_path)?;

        let deadline = Instant::now() + Duration::from_millis(1800);
        let mut last_error = None;
        while Instant::now() < deadline {
            match self.send_render_to_existing_viewer(
                manifest_path,
                output_path,
                width,
                height,
                camera,
            ) {
                Ok(()) => return Ok(()),
                Err(error) => {
                    last_error = Some(error);
                    thread::sleep(Duration::from_millis(50));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            Error::Internal("resident model-insight viewer did not become ready".to_string())
        }))
    }

    fn launch_resident_viewer(&self, manifest_path: &PathBuf) -> Result<String> {
        let now = Instant::now();
        {
            let mut started_at = self.resident_viewer_start.lock();
            if started_at.is_some_and(|started_at| {
                now.saturating_duration_since(started_at) < Duration::from_secs(5)
            }) {
                return Ok(self.viewer_executable_hint().unwrap_or_default());
            }
            *started_at = Some(now);
        }

        let executable_path = external_tools::find_model_insight_cli().ok_or_else(|| {
            Error::Internal(format!(
                "model-insight not found. Place it under: {}",
                external_tools::model_insight_status().expected_path
            ))
        })?;

        if let Err(error) = Command::new(&executable_path)
            .args(["view", "--resident", "--request"])
            .arg(manifest_path)
            .spawn()
        {
            *self.resident_viewer_start.lock() = None;
            return Err(Error::Internal(format!(
                "failed to start model-insight: {error}"
            )));
        }

        Ok(executable_path.to_string_lossy().to_string())
    }

    fn viewer_executable_hint(&self) -> Option<String> {
        external_tools::find_model_insight_cli().map(|path| path.to_string_lossy().to_string())
    }
}

struct ModelInsightRequest {
    session_id: String,
    manifest_path: PathBuf,
    rpc_addr: String,
}

fn run_broker(listener: TcpListener, state: BrokerState) {
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let state = state.clone();
                thread::spawn(move || handle_broker_stream(stream, state));
            }
            Err(error) => log::warn!("model-insight broker accept failed: {error}"),
        }
    }
}

fn handle_broker_stream(mut stream: TcpStream, state: BrokerState) {
    let request = match read_json_packet::<JsonRpcRequest>(&mut stream) {
        Ok(request) => request,
        Err(error) => {
            log::warn!("model-insight JSON-RPC read failed: {error}");
            return;
        }
    };

    let id = request.id;
    let response = match handle_broker_request(request, &state) {
        Ok(result) => JsonRpcResponse {
            jsonrpc: "2.0",
            id,
            result: Some(result),
            error: None,
        },
        Err(error) => {
            log::warn!("model-insight asset request failed: {error}");
            JsonRpcResponse {
                jsonrpc: "2.0",
                id,
                result: None,
                error: Some(JsonRpcError {
                    code: -32000,
                    message: error.to_string(),
                }),
            }
        }
    };

    if let Err(error) = write_json_packet(&mut stream, &response) {
        log::warn!("model-insight JSON-RPC write failed: {error}");
    }
}

fn read_json_packet<T: for<'de> Deserialize<'de>>(stream: &mut TcpStream) -> Result<T> {
    let mut len = [0u8; 4];
    stream.read_exact(&mut len)?;
    let len = u32::from_be_bytes(len) as usize;
    if len > 8 * 1024 * 1024 {
        return Err(Error::Internal(format!("JSON-RPC packet too large: {len}")));
    }
    let mut body = vec![0u8; len];
    stream.read_exact(&mut body)?;
    serde_json::from_slice(&body).map_err(|error| Error::Internal(error.to_string()))
}

fn write_json_packet<T: Serialize>(stream: &mut TcpStream, value: &T) -> Result<()> {
    let body = serde_json::to_vec(value).map_err(|error| Error::Internal(error.to_string()))?;
    let len = u32::try_from(body.len())
        .map_err(|_| Error::Internal("JSON-RPC packet too large".to_string()))?;
    stream.write_all(&len.to_be_bytes())?;
    stream.write_all(&body)?;
    Ok(())
}

fn handle_broker_request(request: JsonRpcRequest, state: &BrokerState) -> Result<Value> {
    match request.method.as_str() {
        "materializeAsset" => serde_json::to_value(handle_asset_request(request.params, state)?)
            .map_err(|error| Error::Internal(error.to_string())),
        "registerViewer" => serde_json::to_value(handle_register_viewer(request.params, state)?)
            .map_err(|error| Error::Internal(error.to_string())),
        other => Err(Error::Internal(format!(
            "unsupported JSON-RPC method: {other}"
        ))),
    }
}

fn handle_register_viewer(params: Value, state: &BrokerState) -> Result<RegisterViewerResponse> {
    let params: RegisterViewerParams =
        serde_json::from_value(params).map_err(|error| Error::Internal(error.to_string()))?;
    let session = state
        .sessions
        .lock()
        .get(&params.session_id)
        .cloned()
        .ok_or_else(|| {
            Error::Internal(format!(
                "unknown model-insight session: {}",
                params.session_id
            ))
        })?;

    if params.token != session.token {
        return Err(Error::Internal(
            "invalid model-insight registration token".to_string(),
        ));
    }

    *state.viewer.lock() = Some(ModelInsightInstance {
        control_addr: params.control_addr,
    });
    Ok(RegisterViewerResponse { registered: true })
}

fn handle_asset_request(params: Value, state: &BrokerState) -> Result<AssetMaterializeResponse> {
    let params: AssetMaterializeParams =
        serde_json::from_value(params).map_err(|error| Error::Internal(error.to_string()))?;

    let session = state
        .sessions
        .lock()
        .get(&params.session_id)
        .cloned()
        .ok_or_else(|| {
            Error::Internal(format!(
                "unknown model-insight session: {}",
                params.session_id
            ))
        })?;

    if params.token != session.token {
        return Err(Error::Internal(
            "invalid model-insight asset token".to_string(),
        ));
    }

    Ok(match params.kind.as_str() {
        "mesh" => materialize_mesh(state.pak_service, &session)?,
        "mdf" => materialize_mdf(state.pak_service, &session)?,
        "texture" => {
            let texture_path = params
                .path
                .ok_or_else(|| Error::Internal("missing texture path".to_string()))?;
            let base_entry_path = params.base.as_deref().unwrap_or(&session.mesh_entry_path);
            materialize_texture(state.pak_service, &session, base_entry_path, &texture_path)?
        }
        other => return Err(Error::Internal(format!("unsupported asset kind: {other}"))),
    })
}

fn materialize_mesh(
    pak_service: &PakService,
    session: &ModelInsightSession,
) -> Result<AssetMaterializeResponse> {
    materialize_hash(
        pak_service,
        session,
        session.mesh_hash,
        session.mesh_belongs_to,
        &session.mesh_entry_path,
    )
}

fn materialize_mdf(
    pak_service: &PakService,
    session: &ModelInsightSession,
) -> Result<AssetMaterializeResponse> {
    let resolved = find_adjacent_mdf_entry(
        pak_service,
        &session.mesh_entry_path,
        session.mesh_belongs_to,
    )?;
    materialize_hash(
        pak_service,
        session,
        resolved.hash,
        resolved.belongs_to,
        &resolved.entry_path,
    )
}

fn materialize_texture(
    pak_service: &PakService,
    session: &ModelInsightSession,
    base_entry_path: &str,
    texture_path: &str,
) -> Result<AssetMaterializeResponse> {
    let resolved = find_texture_entry(
        pak_service,
        base_entry_path,
        texture_path,
        session.mesh_belongs_to,
    )?;
    materialize_hash(
        pak_service,
        session,
        resolved.hash,
        resolved.belongs_to,
        &resolved.entry_path,
    )
}

fn materialize_hash(
    pak_service: &PakService,
    session: &ModelInsightSession,
    hash: u64,
    belongs_to: Option<PakId>,
    entry_path: &str,
) -> Result<AssetMaterializeResponse> {
    let output_path = session.temp_dir.join(format!(
        "{hash:016X}-{}",
        sanitize_file_name(file_name(entry_path))
    ));

    if !output_path.exists() {
        pak_service.unpack_file_by_hash(hash, belongs_to, &output_path)?;
    }

    Ok(AssetMaterializeResponse {
        local_path: output_path.to_string_lossy().to_string(),
        entry_path: normalize_entry_path(entry_path),
    })
}

#[derive(Debug, Clone)]
struct ResolvedPakEntry {
    hash: u64,
    entry_path: String,
    belongs_to: Option<PakId>,
    version: u64,
}

fn find_adjacent_mdf_entry(
    pak_service: &PakService,
    mesh_entry_path: &str,
    preferred_pak: Option<PakId>,
) -> Result<ResolvedPakEntry> {
    let mesh_entry_path = normalize_entry_path(mesh_entry_path);
    let Some((directory, name)) = split_parent_name(&mesh_entry_path) else {
        return Err(Error::PakEntryNotFound(mesh_entry_path));
    };
    let Some((base, _)) = name.split_once(".mesh.") else {
        return Err(Error::PakEntryNotFound(format!(
            "mdf for {mesh_entry_path}"
        )));
    };

    let candidates = collect_named_candidates(pak_service, |path| {
        let Some((candidate_dir, candidate_name)) = split_parent_name(path) else {
            return false;
        };
        candidate_dir == directory
            && candidate_name.starts_with(base)
            && candidate_name.contains(".mdf2.")
    })?;

    select_loaded_candidate(pak_service, candidates, preferred_pak)
        .ok_or_else(|| Error::PakEntryNotFound(format!("mdf for {mesh_entry_path}")))
}

fn find_texture_entry(
    pak_service: &PakService,
    base_entry_path: &str,
    texture_path: &str,
    preferred_pak: Option<PakId>,
) -> Result<ResolvedPakEntry> {
    let stems = texture_lookup_stems(base_entry_path, texture_path);
    let candidates = collect_named_candidates(pak_service, |path| {
        let normalized = path.trim_end_matches(".tex").replace('\\', "/");
        stems.iter().any(|stem| {
            normalized
                .to_ascii_lowercase()
                .starts_with(&format!("{}.tex.", stem.to_ascii_lowercase()))
        })
    })?;

    select_loaded_candidate(pak_service, candidates, preferred_pak)
        .ok_or_else(|| Error::PakEntryNotFound(format!("texture {texture_path}")))
}

fn collect_named_candidates(
    pak_service: &PakService,
    matches_path: impl Fn(&str) -> bool,
) -> Result<Vec<ResolvedPakEntry>> {
    let pak_group = pak_service.pak_group();
    let pak_group = pak_group.lock();
    let Some(file_name_table) = pak_group.file_name_table() else {
        return Err(Error::MissingFileList);
    };

    Ok(file_name_table
        .file_names()
        .filter_map(|(hash, path)| {
            let path = path.to_string().ok()?.replace('\\', "/");
            matches_path(&path).then(|| ResolvedPakEntry {
                hash: *hash,
                version: version_suffix(&path),
                entry_path: path,
                belongs_to: None,
            })
        })
        .collect())
}

fn select_loaded_candidate(
    pak_service: &PakService,
    candidates: Vec<ResolvedPakEntry>,
    preferred_pak: Option<PakId>,
) -> Option<ResolvedPakEntry> {
    let pak_group = pak_service.pak_group();
    let pak_group = pak_group.lock();
    if let Some(pak_id) = preferred_pak
        && let Some(pak) = pak_group.get_pak(&pak_id)
        && let Some(candidate) = newest_candidate_in_pak(&candidates, pak, pak_id)
    {
        return Some(candidate);
    }

    pak_group
        .paks()
        .iter()
        .rev()
        .find_map(|pak| newest_candidate_in_pak(&candidates, pak, pak.id))
}

fn newest_candidate_in_pak(
    candidates: &[ResolvedPakEntry],
    pak: &Pak,
    pak_id: PakId,
) -> Option<ResolvedPakEntry> {
    candidates
        .iter()
        .filter(|candidate| pak_has_entry(pak, candidate.hash))
        .max_by_key(|candidate| candidate.version)
        .map(|candidate| {
            let mut candidate = candidate.clone();
            candidate.belongs_to = Some(pak_id);
            candidate
        })
}

fn pak_has_entry(pak: &Pak, hash: u64) -> bool {
    pak.pakfile
        .metadata()
        .entries()
        .iter()
        .any(|entry| entry.hash() == hash)
}

fn texture_lookup_stems(base_entry_path: &str, texture_path: &str) -> Vec<String> {
    let normalized = normalize_texture_path(texture_path);
    let mut stems = vec![normalized.clone()];
    if !normalized.starts_with("natives/")
        && let Some(natives_root) = natives_root(base_entry_path)
    {
        stems.push(format!("{natives_root}/{normalized}"));
    }
    stems.sort();
    stems.dedup();
    stems
}

fn normalize_texture_path(texture_path: &str) -> String {
    texture_path
        .trim_start_matches('@')
        .replace('\\', "/")
        .trim_end_matches(".tex")
        .trim_start_matches('/')
        .to_string()
}

fn natives_root(entry_path: &str) -> Option<String> {
    let parts: Vec<_> = normalize_entry_path(entry_path)
        .split('/')
        .map(str::to_string)
        .collect();
    let index = parts
        .iter()
        .position(|part| part.eq_ignore_ascii_case("natives"))?;
    parts.get(index + 1)?;
    Some(parts[..=index + 1].join("/"))
}

fn split_parent_name(path: &str) -> Option<(&str, &str)> {
    path.rsplit_once('/').or(Some(("", path)))
}

fn file_name(path: &str) -> &str {
    split_parent_name(path)
        .map(|(_, name)| name)
        .unwrap_or(path)
}

fn version_suffix(path: &str) -> u64 {
    path.rsplit_once('.')
        .and_then(|(_, suffix)| suffix.parse::<u64>().ok())
        .unwrap_or(0)
}

fn normalize_entry_path(path: &str) -> String {
    path.replace('\\', "/").trim_start_matches('/').to_string()
}

fn sanitize_file_name(name: &str) -> String {
    name.chars()
        .map(|ch| match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            other => other,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_mdf_texture_path() {
        assert_eq!(
            normalize_texture_path("@Art\\Model\\foo\\bar_ALBD.tex"),
            "Art/Model/foo/bar_ALBD"
        );
    }

    #[test]
    fn builds_texture_stems_with_mesh_native_root() {
        let stems = texture_lookup_stems(
            "natives/STM/Art/Model/ch00/body.mesh.2109148288",
            "@Art/Model/ch00/body_ALBD.tex",
        );

        assert!(stems.contains(&"Art/Model/ch00/body_ALBD".to_string()));
        assert!(stems.contains(&"natives/STM/Art/Model/ch00/body_ALBD".to_string()));
    }

    #[test]
    fn parses_version_suffix() {
        assert_eq!(version_suffix("foo/bar.tex.230110883"), 230110883);
        assert_eq!(version_suffix("foo/bar.tex"), 0);
    }
}
