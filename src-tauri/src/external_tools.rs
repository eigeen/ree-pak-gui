use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{
    error::{Error, Result},
    get_local_dir, utility,
};

const EXTENSIONS_DIR_NAME: &str = "extensions";
const VGMSTREAM_EXTENSION_NAME: &str = "vgmstream";
const MODEL_INSIGHT_EXTENSION_NAME: &str = "model-insight";

pub fn extension_dir(name: &str) -> PathBuf {
    get_local_dir().join(EXTENSIONS_DIR_NAME).join(name)
}

pub fn platform_arch_dir_name() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VgmstreamStatus {
    pub installed: bool,
    pub platform: String,
    pub arch: String,
    pub asset_name: Option<String>,
    pub install_dir: String,
    pub expected_path: String,
    pub executable_path: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInsightStatus {
    pub installed: bool,
    pub platform: String,
    pub arch: String,
    pub install_dir: String,
    pub expected_path: String,
    pub executable_path: Option<String>,
}

pub fn vgmstream_cli_candidates() -> Vec<PathBuf> {
    let root = extension_dir(VGMSTREAM_EXTENSION_NAME);
    let platform_dir = root.join(platform_arch_dir_name());

    let mut candidates = vgmstream_platform_cli_candidates(&platform_dir);

    candidates.push(root.join(vgmstream_cli_exe_name()));
    if cfg!(target_os = "windows") {
        candidates.push(root.join("vgmstream-win64").join(vgmstream_cli_exe_name()));
    }

    candidates
}

pub fn find_vgmstream_cli() -> Option<PathBuf> {
    vgmstream_cli_candidates()
        .into_iter()
        .find(|path| path.is_file())
}

pub fn vgmstream_status() -> VgmstreamStatus {
    let install_dir = vgmstream_platform_dir();
    let candidates = vgmstream_platform_cli_candidates(&install_dir);
    let expected_path = candidates
        .first()
        .cloned()
        .unwrap_or_else(|| install_dir.join(vgmstream_cli_exe_name()));
    let executable_path = vgmstream_cli_candidates()
        .into_iter()
        .find(|path| path.is_file());

    VgmstreamStatus {
        installed: executable_path.is_some(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        asset_name: vgmstream_download_asset_name().map(str::to_string),
        install_dir: install_dir.to_string_lossy().to_string(),
        expected_path: expected_path.to_string_lossy().to_string(),
        executable_path: executable_path.map(|path| path.to_string_lossy().to_string()),
    }
}

pub fn model_insight_cli_candidates() -> Vec<PathBuf> {
    let root = extension_dir(MODEL_INSIGHT_EXTENSION_NAME);
    let platform_dir = root.join(platform_arch_dir_name());
    let mut candidates = vec![platform_dir.join(model_insight_cli_exe_name())];

    candidates.push(root.join(model_insight_cli_exe_name()));
    if cfg!(target_os = "windows") {
        candidates.push(
            root.join("model-insight")
                .join(model_insight_cli_exe_name()),
        );
    }

    candidates
}

pub fn find_model_insight_cli() -> Option<PathBuf> {
    model_insight_cli_candidates()
        .into_iter()
        .find(|path| path.is_file())
}

pub fn model_insight_status() -> ModelInsightStatus {
    let install_dir = extension_dir(MODEL_INSIGHT_EXTENSION_NAME).join(platform_arch_dir_name());
    let expected_path = install_dir.join(model_insight_cli_exe_name());
    let executable_path = find_model_insight_cli();

    ModelInsightStatus {
        installed: executable_path.is_some(),
        platform: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        install_dir: install_dir.to_string_lossy().to_string(),
        expected_path: expected_path.to_string_lossy().to_string(),
        executable_path: executable_path.map(|path| path.to_string_lossy().to_string()),
    }
}

pub fn install_vgmstream_from_archive(archive_path: impl Into<PathBuf>) -> Result<VgmstreamStatus> {
    if vgmstream_download_asset_name().is_none() {
        return Err(Error::Internal(format!(
            "Unsupported platform for vgmstream: {}-{}",
            std::env::consts::OS,
            std::env::consts::ARCH
        )));
    }

    let archive_path = archive_path.into();
    let install_dir = vgmstream_platform_dir();
    if install_dir.exists() {
        std::fs::remove_dir_all(&install_dir)?;
    }
    std::fs::create_dir_all(&install_dir)?;

    let extracted_files = utility::zip_extract_all(&archive_path, &install_dir)?;
    let cli_path = vgmstream_platform_cli_candidates(&install_dir)
        .into_iter()
        .find(|path| path.is_file())
        .ok_or_else(|| {
            Error::Internal(format!(
                "vgmstream-cli not found after extraction. Extracted files: {}",
                extracted_files.join(", ")
            ))
        })?;

    make_executable(&cli_path)?;
    Ok(vgmstream_status())
}

fn vgmstream_platform_dir() -> PathBuf {
    extension_dir(VGMSTREAM_EXTENSION_NAME).join(platform_arch_dir_name())
}

fn vgmstream_platform_cli_candidates(platform_dir: &Path) -> Vec<PathBuf> {
    let exe_name = vgmstream_cli_exe_name();
    let mut candidates = vec![platform_dir.join(exe_name)];

    if cfg!(target_os = "windows") {
        candidates.push(platform_dir.join("vgmstream-win64").join(exe_name));
    }

    candidates
}

fn vgmstream_download_asset_name() -> Option<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("windows", "x86_64") => Some("vgmstream-win64.zip"),
        ("macos", "x86_64" | "aarch64") => Some("vgmstream-mac-cli.zip"),
        ("linux", "x86_64" | "aarch64") => Some("vgmstream-linux-cli.zip"),
        _ => None,
    }
}

fn vgmstream_cli_exe_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "vgmstream-cli.exe"
    } else {
        "vgmstream-cli"
    }
}

fn model_insight_cli_exe_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "model-insight.exe"
    } else {
        "model-insight"
    }
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vgmstream_candidates_prefer_platform_arch_directory() {
        let candidates = vgmstream_cli_candidates();

        assert!(candidates.len() >= 2);
        assert!(
            candidates[0]
                .to_string_lossy()
                .contains(&platform_arch_dir_name())
        );
        assert!(candidates[1].to_string_lossy().contains("vgmstream"));
    }
}
