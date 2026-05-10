use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::OnceLock,
    time::Instant,
};

use serde::{Deserialize, Serialize};

use crate::{
    TEMP_DIR_NAME,
    common::JsSafeHash,
    error::{Error, Result},
    get_local_dir,
    pak::{Pak, PakId},
    service::pak::PakService,
};

static MODEL_INSIGHT_SERVICE: OnceLock<ModelInsightService> = OnceLock::new();

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightLoadMeshAssetsOptions {
    pub hash: JsSafeHash,
    pub belongs_to: Option<PakId>,
    pub entry_path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightMeshAssets {
    pub mesh_entry_path: String,
    pub mesh_file_version: u32,
    pub mesh_data: Vec<u8>,
    pub mdf_entry_path: Option<String>,
    pub mdf_file_version: Option<u32>,
    pub mdf_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightLoadTexturePreviewsOptions {
    pub belongs_to: Option<PakId>,
    pub base_entry_path: String,
    pub texture_paths: Vec<String>,
    pub texture_resolution: Option<ModelTextureResolution>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ModelTextureResolution {
    Standard,
    High,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightTexturePreview {
    pub texture_path: String,
    pub entry_path: String,
    pub preview_path: PathBuf,
    pub preview_data: Vec<u8>,
}

pub struct ModelInsightService {
    pak_service: &'static PakService,
    temp_dir: PathBuf,
}

impl ModelInsightService {
    pub fn initialize() -> Result<&'static Self> {
        let temp_dir = get_local_dir().join(TEMP_DIR_NAME).join("model-insight");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(MODEL_INSIGHT_SERVICE.get_or_init(|| Self {
            pak_service: PakService::get(),
            temp_dir,
        }))
    }

    pub fn get() -> &'static Self {
        MODEL_INSIGHT_SERVICE.get().unwrap()
    }

    pub fn load_mesh_assets(
        &self,
        options: ModelInsightLoadMeshAssetsOptions,
    ) -> Result<ModelInsightMeshAssets> {
        let mesh_entry_path = normalize_entry_path(&options.entry_path);
        let mesh_file_version = checked_version_suffix(&mesh_entry_path, "mesh")?;
        let asset_dir = self.temp_dir.join("wasm-assets");
        std::fs::create_dir_all(&asset_dir)?;

        let mesh_data = self.materialize_hash_bytes(
            options.hash.hash_u64(),
            options.belongs_to,
            &mesh_entry_path,
            &asset_dir,
        )?;

        let mdf =
            match find_adjacent_mdf_entry(self.pak_service, &mesh_entry_path, options.belongs_to) {
                Ok(resolved) => {
                    let mdf_file_version = checked_version_suffix(&resolved.entry_path, "mdf2")?;
                    let mdf_data = self.materialize_hash_bytes(
                        resolved.hash,
                        resolved.belongs_to,
                        &resolved.entry_path,
                        &asset_dir,
                    )?;
                    Some((resolved.entry_path, mdf_file_version, mdf_data))
                }
                Err(Error::PakEntryNotFound(missing)) => {
                    log::warn!(
                        "model insight mdf not found: mesh={mesh_entry_path} missing={missing}"
                    );
                    None
                }
                Err(error) => return Err(error),
            };

        Ok(ModelInsightMeshAssets {
            mesh_entry_path,
            mesh_file_version,
            mesh_data,
            mdf_entry_path: mdf.as_ref().map(|(entry_path, _, _)| entry_path.clone()),
            mdf_file_version: mdf.as_ref().map(|(_, file_version, _)| *file_version),
            mdf_data: mdf.map(|(_, _, data)| data),
        })
    }

    pub fn load_texture_previews(
        &self,
        options: ModelInsightLoadTexturePreviewsOptions,
    ) -> Result<Vec<ModelInsightTexturePreview>> {
        let base_entry_path = normalize_entry_path(&options.base_entry_path);
        let texture_resolution = options
            .texture_resolution
            .unwrap_or(ModelTextureResolution::Standard);
        let asset_dir = self.temp_dir.join("wasm-assets");
        std::fs::create_dir_all(&asset_dir)?;
        let resolve_context_started_at = Instant::now();
        let resolve_context = TextureResolveContext::new(self.pak_service)?;
        log::info!(
            "model insight texture resolve index ready: tex_bases={} paks={} elapsed={} ms",
            resolve_context.tex_candidates_by_base.len(),
            resolve_context.loaded_hashes_by_pak.len(),
            resolve_context_started_at.elapsed().as_millis()
        );

        let mut seen = HashSet::new();
        let mut previews = Vec::new();
        for texture_path in options.texture_paths {
            if !seen.insert(texture_path.clone()) {
                continue;
            }

            let started_at = Instant::now();
            let resolve_started_at = Instant::now();
            let resolved = match resolve_texture_entry(
                &resolve_context,
                &base_entry_path,
                &texture_path,
                &options.belongs_to,
                texture_resolution,
            ) {
                Ok(resolved) => resolved,
                Err(error) => {
                    log::warn!(
                        "model insight texture skipped: base={} texture={} error={}",
                        base_entry_path,
                        texture_path,
                        error
                    );
                    continue;
                }
            };
            let resolve_elapsed_ms = resolve_started_at.elapsed().as_millis();

            let raw_path =
                self.materialized_asset_path(resolved.hash, &resolved.entry_path, &asset_dir);
            let raw_cached = raw_path.exists();
            if !raw_path.exists() {
                self.pak_service.unpack_file_by_hash(
                    resolved.hash,
                    resolved.belongs_to,
                    &raw_path,
                )?;
            }

            let preview_lod = texture_preview_lod(texture_resolution, &resolved.entry_path);
            let preview_path = asset_dir.join(format!(
                "{:016X}-lod{}-{}.png",
                resolved.hash,
                preview_lod,
                sanitize_file_name(&file_stem(&resolved.entry_path))
            ));
            let preview_cached = preview_path.exists();
            if !preview_cached {
                let convert_started_at = Instant::now();
                if let Err(error) =
                    crate::service::preview::tex_to_png_lod(&raw_path, &preview_path, preview_lod)
                {
                    log::warn!(
                        "model insight texture conversion skipped: entry={} error={}",
                        resolved.entry_path,
                        error
                    );
                    continue;
                }
                log::info!(
                    "model insight texture converted: entry={} elapsed={} ms",
                    resolved.entry_path,
                    convert_started_at.elapsed().as_millis()
                );
            }

            let read_started_at = Instant::now();
            let preview_data = std::fs::read(&preview_path)?;
            log::info!(
                "model insight texture preview ready: texture={} entry={} lod={} raw_cached={} preview_cached={} bytes={} resolve_elapsed={} ms read_elapsed={} ms total_elapsed={} ms",
                texture_path,
                resolved.entry_path,
                preview_lod,
                raw_cached,
                preview_cached,
                preview_data.len(),
                resolve_elapsed_ms,
                read_started_at.elapsed().as_millis(),
                started_at.elapsed().as_millis()
            );

            previews.push(ModelInsightTexturePreview {
                texture_path,
                entry_path: resolved.entry_path,
                preview_data,
                preview_path,
            });
        }

        Ok(previews)
    }

    fn materialize_hash_bytes(
        &self,
        hash: u64,
        belongs_to: Option<PakId>,
        entry_path: &str,
        asset_dir: &std::path::Path,
    ) -> Result<Vec<u8>> {
        let output_path = self.materialized_asset_path(hash, entry_path, asset_dir);

        if !output_path.exists() {
            self.pak_service
                .unpack_file_by_hash(hash, belongs_to, &output_path)?;
        }

        std::fs::read(&output_path).map_err(Into::into)
    }

    fn materialized_asset_path(
        &self,
        hash: u64,
        entry_path: &str,
        asset_dir: &std::path::Path,
    ) -> PathBuf {
        asset_dir.join(format!(
            "{hash:016X}-{}",
            sanitize_file_name(file_name(entry_path))
        ))
    }
}

#[derive(Debug, Clone)]
struct ResolvedPakEntry {
    hash: u64,
    entry_path: String,
    belongs_to: Option<PakId>,
    version: u64,
}

struct TextureResolveContext {
    tex_candidates_by_base: HashMap<String, Vec<ResolvedPakEntry>>,
    loaded_hashes_by_pak: Vec<(PakId, HashSet<u64>)>,
}

impl TextureResolveContext {
    fn new(pak_service: &PakService) -> Result<Self> {
        let pak_group = pak_service.pak_group();
        let pak_group = pak_group.lock();
        let Some(file_name_table) = pak_group.file_name_table() else {
            return Err(Error::MissingFileList);
        };

        let mut tex_candidates_by_base = HashMap::<String, Vec<ResolvedPakEntry>>::new();
        for (hash, path) in file_name_table.file_names() {
            let Some(path) = path.to_string().ok().map(|path| path.replace('\\', "/")) else {
                continue;
            };
            let normalized = normalize_entry_path(&path);
            let lowered = normalized.to_ascii_lowercase();

            if let Some((base, _)) = lowered.split_once(".tex.") {
                tex_candidates_by_base
                    .entry(base.to_string())
                    .or_default()
                    .push(ResolvedPakEntry {
                        hash: *hash,
                        version: version_suffix(&normalized),
                        entry_path: normalized,
                        belongs_to: None,
                    });
            }
        }

        let loaded_hashes_by_pak = pak_group
            .paks()
            .iter()
            .map(|pak| {
                (
                    pak.id,
                    pak.pakfile
                        .metadata()
                        .entries()
                        .iter()
                        .map(|entry| entry.hash())
                        .collect::<HashSet<_>>(),
                )
            })
            .collect();

        Ok(Self {
            tex_candidates_by_base,
            loaded_hashes_by_pak,
        })
    }

    fn candidates_for(&self, path_without_version: &str) -> Vec<ResolvedPakEntry> {
        let candidate = normalize_entry_path(path_without_version).to_ascii_lowercase();
        self.tex_candidates_by_base
            .get(&candidate)
            .cloned()
            .unwrap_or_default()
    }
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

fn resolve_texture_entry(
    context: &TextureResolveContext,
    base_entry_path: &str,
    texture_path: &str,
    preferred_pak: &Option<PakId>,
    texture_resolution: ModelTextureResolution,
) -> Result<ResolvedPakEntry> {
    let candidates = texture_entry_candidates(base_entry_path, texture_path, texture_resolution);
    for candidate in candidates {
        let matches = context.candidates_for(&candidate);
        if let Some(resolved) =
            select_loaded_candidate_from_hashes(context, matches, *preferred_pak)
        {
            return Ok(resolved);
        }
    }

    Err(Error::PakEntryNotFound(format!(
        "texture {texture_path} for {base_entry_path}"
    )))
}

fn texture_entry_candidates(
    base_entry_path: &str,
    texture_path: &str,
    texture_resolution: ModelTextureResolution,
) -> Vec<String> {
    let normalized = normalize_material_texture_path(texture_path);
    if normalized.is_empty() {
        return Vec::new();
    }

    let mut candidates = Vec::new();
    push_unique_candidate(
        &mut candidates,
        normalized.trim_start_matches('/').to_string(),
    );

    if !normalized.starts_with('/') {
        if let Some(root) = natives_root_entry(base_entry_path) {
            push_unique_candidate(&mut candidates, join_entry_path(&root, &normalized));
        }
        if let Some((parent, _)) = split_parent_name(base_entry_path)
            && !parent.is_empty()
        {
            push_unique_candidate(&mut candidates, join_entry_path(parent, &normalized));
        }
    }

    let standard_candidates = candidates.clone();
    let mut streaming_candidates = Vec::new();
    for candidate in &standard_candidates {
        if let Some(streaming_candidate) = streaming_texture_candidate(candidate) {
            push_unique_candidate(&mut streaming_candidates, streaming_candidate);
        }
    }

    candidates.clear();
    let ordered_candidate_groups = match texture_resolution {
        ModelTextureResolution::High => [&streaming_candidates, &standard_candidates],
        ModelTextureResolution::Standard => [&standard_candidates, &streaming_candidates],
    };
    for group in ordered_candidate_groups {
        for candidate in group {
            push_unique_candidate(&mut candidates, candidate.clone());
        }
    }

    candidates
}

fn normalize_material_texture_path(texture_path: &str) -> String {
    let mut normalized = texture_path
        .trim()
        .trim_start_matches('@')
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    if normalized.to_ascii_lowercase().ends_with(".tex") {
        normalized.truncate(normalized.len().saturating_sub(4));
    }
    normalized
}

fn natives_root_entry(path: &str) -> Option<String> {
    let components = path
        .split('/')
        .filter(|component| !component.is_empty())
        .collect::<Vec<_>>();
    components
        .iter()
        .position(|component| component.eq_ignore_ascii_case("natives"))
        .and_then(|index| {
            (index + 1 < components.len()).then(|| components[..=index + 1].join("/"))
        })
}

fn streaming_texture_candidate(path: &str) -> Option<String> {
    let mut components = path
        .split('/')
        .filter(|component| !component.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    let natives_index = components
        .iter()
        .position(|component| component.eq_ignore_ascii_case("natives"))?;
    let streaming_index = natives_index + 2;
    if streaming_index > components.len() {
        return None;
    }
    if components
        .get(streaming_index)
        .is_some_and(|component| component.eq_ignore_ascii_case("streaming"))
    {
        return Some(components.join("/"));
    }
    components.insert(streaming_index, "streaming".to_string());
    Some(components.join("/"))
}

fn texture_preview_lod(texture_resolution: ModelTextureResolution, entry_path: &str) -> usize {
    match texture_resolution {
        ModelTextureResolution::High => 0,
        ModelTextureResolution::Standard if is_streaming_entry_path(entry_path) => 2,
        ModelTextureResolution::Standard => 0,
    }
}

fn is_streaming_entry_path(path: &str) -> bool {
    path.split('/')
        .filter(|component| !component.is_empty())
        .any(|component| component.eq_ignore_ascii_case("streaming"))
}

fn join_entry_path(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        return child.trim_start_matches('/').to_string();
    }
    format!(
        "{}/{}",
        parent.trim_end_matches('/'),
        child.trim_start_matches('/')
    )
}

fn push_unique_candidate(candidates: &mut Vec<String>, candidate: String) {
    if !candidate.is_empty() && !candidates.iter().any(|existing| existing == &candidate) {
        candidates.push(candidate);
    }
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

fn select_loaded_candidate_from_hashes(
    context: &TextureResolveContext,
    candidates: Vec<ResolvedPakEntry>,
    preferred_pak: Option<PakId>,
) -> Option<ResolvedPakEntry> {
    if let Some(pak_id) = preferred_pak
        && let Some((_, hashes)) = context
            .loaded_hashes_by_pak
            .iter()
            .find(|(candidate_pak_id, _)| *candidate_pak_id == pak_id)
        && let Some(candidate) = newest_candidate_in_hash_set(&candidates, hashes, pak_id)
    {
        return Some(candidate);
    }

    context
        .loaded_hashes_by_pak
        .iter()
        .rev()
        .find_map(|(pak_id, hashes)| newest_candidate_in_hash_set(&candidates, hashes, *pak_id))
}

fn newest_candidate_in_hash_set(
    candidates: &[ResolvedPakEntry],
    hashes: &HashSet<u64>,
    pak_id: PakId,
) -> Option<ResolvedPakEntry> {
    candidates
        .iter()
        .filter(|candidate| hashes.contains(&candidate.hash))
        .max_by_key(|candidate| candidate.version)
        .map(|candidate| {
            let mut candidate = candidate.clone();
            candidate.belongs_to = Some(pak_id);
            candidate
        })
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

fn split_parent_name(path: &str) -> Option<(&str, &str)> {
    path.rsplit_once('/').or(Some(("", path)))
}

fn file_name(path: &str) -> &str {
    split_parent_name(path)
        .map(|(_, name)| name)
        .unwrap_or(path)
}

fn file_stem(path: &str) -> String {
    let name = file_name(path);
    name.split_once(".tex.")
        .map(|(stem, _)| stem.to_string())
        .unwrap_or_else(|| name.to_string())
}

fn version_suffix(path: &str) -> u64 {
    path.rsplit_once('.')
        .and_then(|(_, suffix)| suffix.parse::<u64>().ok())
        .unwrap_or(0)
}

fn checked_version_suffix(path: &str, kind: &str) -> Result<u32> {
    u32::try_from(version_suffix(path)).map_err(|_| {
        Error::Internal(format!(
            "{kind} file version suffix is too large for wasm API: {path}"
        ))
    })
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
    fn parses_version_suffix() {
        assert_eq!(version_suffix("foo/bar.tex.230110883"), 230110883);
        assert_eq!(version_suffix("foo/bar.tex"), 0);
    }

    #[test]
    fn resolves_model_texture_candidates_from_natives_root() {
        let candidates = texture_entry_candidates(
            "natives/STM/Art/Model/ch000/foo.mesh.2109148288",
            "@Art/Model/ch000/textures/foo_ALBD.tex",
            ModelTextureResolution::Standard,
        );
        assert_eq!(
            candidates,
            vec![
                "Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/Art/Model/ch000/Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/streaming/Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/streaming/Art/Model/ch000/Art/Model/ch000/textures/foo_ALBD"
                    .to_string()
            ]
        );
    }

    #[test]
    fn resolves_high_model_texture_candidates_from_streaming_root() {
        let candidates = texture_entry_candidates(
            "natives/STM/Art/Model/ch000/foo.mesh.2109148288",
            "@Art/Model/ch000/textures/foo_ALBD.tex",
            ModelTextureResolution::High,
        );
        assert_eq!(
            candidates,
            vec![
                "natives/STM/streaming/Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/streaming/Art/Model/ch000/Art/Model/ch000/textures/foo_ALBD"
                    .to_string(),
                "Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/Art/Model/ch000/textures/foo_ALBD".to_string(),
                "natives/STM/Art/Model/ch000/Art/Model/ch000/textures/foo_ALBD".to_string()
            ]
        );
    }

    #[test]
    fn resolves_high_model_texture_candidates_with_any_natives_platform() {
        let candidates = texture_entry_candidates(
            "natives/X64/Art/Model/ch000/foo.mesh.2109148288",
            "@Art/Model/ch000/textures/foo_ALBD.tex",
            ModelTextureResolution::High,
        );
        assert!(
            candidates
                .contains(&"natives/X64/streaming/Art/Model/ch000/textures/foo_ALBD".to_string())
        );
    }

    #[test]
    fn inserts_streaming_after_natives_platform() {
        assert_eq!(
            streaming_texture_candidate("natives/STM/Art/foo_ALBD"),
            Some("natives/STM/streaming/Art/foo_ALBD".to_string())
        );
        assert_eq!(
            streaming_texture_candidate("natives/STM/streaming/Art/foo_ALBD"),
            Some("natives/STM/streaming/Art/foo_ALBD".to_string())
        );
        assert_eq!(streaming_texture_candidate("Art/foo_ALBD"), None);
    }

    #[test]
    fn standard_texture_preview_uses_lower_lod_for_streaming_fallback() {
        assert_eq!(
            texture_preview_lod(
                ModelTextureResolution::Standard,
                "natives/STM/streaming/Art/foo_ALBD.tex.241106027"
            ),
            2
        );
        assert_eq!(
            texture_preview_lod(
                ModelTextureResolution::High,
                "natives/STM/streaming/Art/foo_ALBD.tex.241106027"
            ),
            0
        );
        assert_eq!(
            texture_preview_lod(
                ModelTextureResolution::Standard,
                "natives/STM/Art/foo_ALBD.tex.241106027"
            ),
            0
        );
    }
}
