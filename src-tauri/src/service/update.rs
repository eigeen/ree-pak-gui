use self_update::{
    Extract,
    backends::github,
    update::{Release, ReleaseAsset, ReleaseUpdate},
    version,
};
use serde::Serialize;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};
use tauri::{AppHandle, ipc::Channel};

const UPDATE_REPO_OWNER: &str = "eigeen";
const UPDATE_REPO_NAME: &str = "ree-pak-gui";
const UPDATE_BIN_NAME: &str = "ree-pak-rs";

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppUpdateInfo {
    version: String,
    name: String,
    date: String,
    release_markdown: Option<String>,
}

impl From<Release> for AppUpdateInfo {
    fn from(release: Release) -> Self {
        Self {
            version: release.version,
            name: release.name,
            date: release.date,
            release_markdown: release.body,
        }
    }
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
pub enum AppUpdateProgressEvent {
    Checking,
    #[serde(rename_all = "camelCase")]
    DownloadStarted {
        total: Option<u64>,
    },
    #[serde(rename_all = "camelCase")]
    DownloadProgress {
        downloaded: u64,
        total: Option<u64>,
    },
    Installing,
    Relaunching,
    Finished {
        version: String,
    },
    UpToDate,
    Error {
        error: String,
    },
}

pub type AppUpdateProgressChannel = Channel<AppUpdateProgressEvent>;

fn build_updater() -> self_update::errors::Result<Box<dyn ReleaseUpdate>> {
    github::Update::configure()
        .repo_owner(UPDATE_REPO_OWNER)
        .repo_name(UPDATE_REPO_NAME)
        .bin_name(UPDATE_BIN_NAME)
        .current_version(env!("CARGO_PKG_VERSION"))
        .show_download_progress(false)
        .show_output(false)
        .no_confirm(true)
        .build()
}

fn select_release(updater: &dyn ReleaseUpdate) -> self_update::errors::Result<Option<Release>> {
    let current_version = updater.current_version();
    let target = updater.target();
    let identifier = updater.identifier();
    let releases = updater.get_latest_releases(&current_version)?;
    let releases = releases
        .into_iter()
        .filter(|release| release.asset_for(&target, identifier.as_deref()).is_some())
        .collect::<Vec<_>>();

    let compatible = releases
        .iter()
        .find(|release| {
            version::bump_is_compatible(&current_version, &release.version).unwrap_or(false)
        })
        .cloned();

    Ok(compatible.or_else(|| releases.into_iter().next()))
}

pub fn check_for_update() -> anyhow::Result<Option<AppUpdateInfo>> {
    let updater = build_updater()?;
    let release = select_release(updater.as_ref())?;

    Ok(release.map(AppUpdateInfo::from))
}

fn download_asset(
    asset: &ReleaseAsset,
    archive_path: &Path,
    channel: &AppUpdateProgressChannel,
) -> anyhow::Result<()> {
    let client = reqwest::blocking::Client::new();
    let mut response = client
        .get(&asset.download_url)
        .header(reqwest::header::USER_AGENT, "ree-pak-rs/self-update")
        .header(reqwest::header::ACCEPT, "application/octet-stream")
        .send()?
        .error_for_status()?;

    let total = response.content_length();
    send_progress(channel, AppUpdateProgressEvent::DownloadStarted { total });

    let mut archive = File::create(archive_path)?;
    let mut downloaded = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];

    loop {
        let read = response.read(&mut buffer)?;
        if read == 0 {
            break;
        }

        archive.write_all(&buffer[..read])?;
        downloaded += read as u64;
        send_progress(
            channel,
            AppUpdateProgressEvent::DownloadProgress { downloaded, total },
        );
    }

    Ok(())
}

fn send_progress(channel: &AppUpdateProgressChannel, event: AppUpdateProgressEvent) {
    if let Err(error) = channel.send(event) {
        log::error!("Failed to send app update progress event: {error}");
    }
}

pub async fn install_update(
    app: AppHandle,
    on_event: AppUpdateProgressChannel,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        let result: Result<(), String> = (|| {
            send_progress(&on_event, AppUpdateProgressEvent::Checking);

            let updater = build_updater().map_err(|error| error.to_string())?;
            let target_release =
                select_release(updater.as_ref()).map_err(|error| error.to_string())?;

            let Some(target_release) = target_release else {
                send_progress(&on_event, AppUpdateProgressEvent::UpToDate);
                return Ok(());
            };

            let target_asset = target_release
                .asset_for(&updater.target(), updater.identifier().as_deref())
                .ok_or_else(|| format!("No asset found for target: `{}`", updater.target()))?;

            let tmp_archive_dir = tempfile::TempDir::new().map_err(|error| error.to_string())?;
            let tmp_archive_path = tmp_archive_dir.path().join(&target_asset.name);
            download_asset(&target_asset, &tmp_archive_path, &on_event)
                .map_err(|error| error.to_string())?;

            send_progress(&on_event, AppUpdateProgressEvent::Installing);

            let bin_path_in_archive = updater.bin_path_in_archive();
            Extract::from_source(&tmp_archive_path)
                .extract_file(tmp_archive_dir.path(), &bin_path_in_archive)
                .map_err(|error| error.to_string())?;

            let new_exe = tmp_archive_dir.path().join(&bin_path_in_archive);
            let bin_install_path = updater.bin_install_path();
            if bin_install_path == std::env::current_exe().map_err(|error| error.to_string())? {
                self_update::self_replace::self_replace(&new_exe)
                    .map_err(|error| error.to_string())?;
            } else {
                self_update::Move::from_source(new_exe.as_ref())
                    .to_dest(bin_install_path.as_ref())
                    .map_err(|error| error.to_string())?;
            }

            send_progress(
                &on_event,
                AppUpdateProgressEvent::Finished {
                    version: target_release.version,
                },
            );
            send_progress(&on_event, AppUpdateProgressEvent::Relaunching);
            app.request_restart();
            Ok(())
        })();

        if let Err(error) = &result {
            send_progress(
                &on_event,
                AppUpdateProgressEvent::Error {
                    error: error.clone(),
                },
            );
        }

        result
    })
    .await
    .map_err(|error| error.to_string())?
}
