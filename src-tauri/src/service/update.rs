use anyhow::{Context, Result};
use parking_lot::Mutex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Digest;

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::time::Duration;
use std::{env, fs};

const METADATA_URL: &[&str] = &[
    "https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/update.json",
    "https://gitee.com/eigeen/ree-pak-gui-update/raw/main/update.json",
];
const PLATFORM: &str = env::consts::OS;
const ARCH: &str = env::consts::ARCH;

static UPDATE_SERVICE: LazyLock<UpdateService> = LazyLock::new(|| {
    #[cfg(feature = "nightly")]
    return UpdateService::new(
        env!("CARGO_PKG_VERSION"),
        env!("GIT_COMMIT_TIME_RFC3339"),
        UpdateChannel::Nightly,
    );
    #[cfg(not(feature = "nightly"))]
    return UpdateService::new(
        env!("CARGO_PKG_VERSION"),
        env!("GIT_COMMIT_TIME_RFC3339"),
        UpdateChannel::Release,
    );
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMetadata {
    pub versions: Vec<UpdateVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateVersion {
    pub version: String,
    pub channel: UpdateChannel,
    pub pub_time: String,
    /// Minimum version required to update to this version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_version: Option<String>,
    pub files: Vec<UpdateFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateFile {
    pub name: String,
    pub size: u64,
    pub sha256: String,
    pub urls: Vec<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UpdateChannel {
    Release,
    Nightly,
}

pub struct ProgramInfo {
    version: semver::Version,
    commit_time: chrono::DateTime<chrono::FixedOffset>,
    channel: UpdateChannel,
}

/// Update Service
pub struct UpdateService {
    client: Client,
    program_info: ProgramInfo,
    metadata: Mutex<Option<UpdateMetadata>>,
}

impl UpdateService {
    fn new(current_version: &str, commit_time_rfc3339: &str, channel: UpdateChannel) -> Self {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            program_info: ProgramInfo {
                version: semver::Version::parse(current_version).expect("Invalid current version"),
                commit_time: chrono::DateTime::parse_from_rfc3339(commit_time_rfc3339).expect("Invalid commit time"),
                channel,
            },
            metadata: Mutex::new(None),
        }
    }

    /// Get global instance of update service.
    pub fn global() -> &'static UpdateService {
        &UPDATE_SERVICE
    }

    /// 检查是否有可用更新
    pub async fn check_update(&self) -> Option<UpdateVersion> {
        let metadata = if let Some(metadata) = self.metadata.lock().as_ref() {
            metadata.clone()
        } else {
            let mt = match self.fetch_metadata().await {
                Ok(metadata) => metadata,
                Err(e) => {
                    log::error!("Failed to fetch update metadata: {}", e);
                    return None;
                }
            };
            self.metadata.lock().replace(mt.clone());
            mt
        };

        // 先检查version，如果为最新版本，再检查pub_time
        let latest_ver = metadata
            .versions
            .iter()
            .filter(|v| v.channel == self.program_info.channel)
            .max_by_key(|v| semver::Version::parse(&v.version).unwrap_or(semver::Version::new(0, 0, 0)));
        let Some(latest_ver) = latest_ver else {
            log::info!("No available updates");
            return None;
        };

        let latest_semver = semver::Version::parse(&latest_ver.version).unwrap_or(semver::Version::new(0, 0, 0));
        if self.program_info.version > latest_semver {
            log::info!("Current version is newer than latest version");
            return None;
        }
        if self.program_info.version < latest_semver {
            log::info!("New version available: {}", latest_ver.version);
            return Some(latest_ver.clone());
        }
        // 发布时间检查
        let Ok(pub_time) = chrono::DateTime::parse_from_rfc3339(&latest_ver.pub_time) else {
            log::error!("Failed to parse publish time: {}", latest_ver.pub_time);
            return None;
        };
        if self.program_info.version == latest_semver && self.program_info.commit_time < pub_time {
            log::info!(
                "New version available: {} (publish time newer: {})",
                latest_ver.version,
                pub_time.to_rfc3339()
            );
            return Some(latest_ver.clone());
        }

        None
    }

    /// 下载更新文件
    pub async fn fetch_update_file(&self, update_version: &UpdateVersion) -> Result<PathBuf> {
        if self.metadata.lock().is_none() {
            return Err(anyhow::anyhow!("Update metadata not available."));
        }
        log::info!(
            "Starting update to {:?} version {} ({})",
            update_version.channel,
            update_version.version,
            update_version.pub_time
        );

        // 下载并校验文件
        let target_file = update_version
            .files
            .iter()
            .find(|f| f.name.contains(PLATFORM) && f.name.contains(ARCH));
        let Some(target_file) = target_file else {
            return Err(anyhow::anyhow!(
                "No matching file found from update metadata for current platform and architecture."
            ));
        };

        // if target file already exists, check sha256
        let target_path = self.get_downloading_dir().join(&target_file.name);
        if target_path.is_file() {
            let mut hasher = sha2::Sha256::new();
            let mut file = File::open(&target_path).context("Failed to open existing file")?;
            std::io::copy(&mut file, &mut hasher).context("Failed to read existing file")?;
            let sha256 = format!("{:x}", hasher.finalize());
            if sha256 == target_file.sha256.to_lowercase() {
                log::info!("Update file already exists, skipping download.");
                return Ok(target_path);
            }
        }

        // try servers
        let mut last_error: Option<anyhow::Error> = None;
        for url in target_file.urls.iter() {
            log::info!("Downloading update from {}", url);
            let response = self.client.get(url).send().await.context("Failed to send request")?;
            if !response.status().is_success() {
                log::warn!("Server returned error status: {}", response.status());
                last_error = Some(anyhow::anyhow!("Server returned error status: {}", response.status()));
                continue;
            }

            let content = response.bytes().await.context("Failed to read response content")?;
            // 校验sha256
            let mut hasher = sha2::Sha256::new();
            hasher.update(&content);
            let sha256 = format!("{:x}", hasher.finalize());
            if sha256 != target_file.sha256.to_lowercase() {
                log::warn!("Checksum mismatch: expected {}, got {}", target_file.sha256, sha256);
                last_error = Some(anyhow::anyhow!(
                    "Checksum mismatch: expected {}, got {}",
                    target_file.sha256,
                    sha256
                ));
                continue;
            }
            // 写入文件
            let mut downloaded_file = File::create(&target_path).context("Failed to create downloading file")?;
            downloaded_file.write_all(&content).context("Failed to write file")?;
            return Ok(target_path);
        }
        if let Some(e) = last_error {
            return Err(e);
        }

        Err(anyhow::anyhow!("All servers failed to provide update file"))
    }

    /// Apply update, wait for restart.
    pub fn perform_update(&self, update_file: impl AsRef<Path>) -> Result<()> {
        let update_file = update_file.as_ref();
        // replace update
        self_replace::self_replace(update_file).context("Failed to replace current binary")?;
        let _ = fs::remove_file(update_file);

        Ok(())
    }

    /// 从服务器获取更新元数据(支持fallback)
    async fn fetch_metadata(&self) -> Result<UpdateMetadata> {
        let mut last_error = None;

        for &url in METADATA_URL {
            match self.try_fetch_metadata(url).await {
                Ok(metadata) => return Ok(metadata),
                Err(e) => {
                    log::warn!("Failed to fetch metadata from {}: {}", url, e);
                    last_error = Some(e);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("Failed to fetch metadata from all servers.")))
    }

    fn get_downloading_dir(&self) -> PathBuf {
        let path = Path::new("downloading");
        if !path.exists() {
            std::fs::create_dir(path).expect("Failed to create downloading directory");
        }
        path.to_path_buf()
    }

    async fn try_fetch_metadata(&self, url: &str) -> Result<UpdateMetadata> {
        let response = self.client.get(url).send().await.context("Failed to send request")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!("Server returned error status: {}", response.status()));
        }

        let metadata = response
            .json::<UpdateMetadata>()
            .await
            .context("Failed to parse metadata")?;

        Ok(metadata)
    }
}
