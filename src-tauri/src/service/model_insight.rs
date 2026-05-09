use std::{path::PathBuf, sync::OnceLock};

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
                Err(Error::PakEntryNotFound(_)) => None,
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

    fn materialize_hash_bytes(
        &self,
        hash: u64,
        belongs_to: Option<PakId>,
        entry_path: &str,
        asset_dir: &std::path::Path,
    ) -> Result<Vec<u8>> {
        let output_path = asset_dir.join(format!(
            "{hash:016X}-{}",
            sanitize_file_name(file_name(entry_path))
        ));

        if !output_path.exists() {
            self.pak_service
                .unpack_file_by_hash(hash, belongs_to, &output_path)?;
        }

        std::fs::read(&output_path).map_err(Into::into)
    }
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
}
