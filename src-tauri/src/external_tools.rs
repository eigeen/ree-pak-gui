use std::path::PathBuf;

use crate::get_local_dir;

const EXTENSIONS_DIR_NAME: &str = "extensions";

pub fn extension_dir(name: &str) -> PathBuf {
    get_local_dir().join(EXTENSIONS_DIR_NAME).join(name)
}

pub fn platform_arch_dir_name() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

pub fn vgmstream_cli_candidates() -> Vec<PathBuf> {
    let root = extension_dir("vgmstream");
    let exe_name = vgmstream_cli_exe_name();
    let platform_dir = root.join(platform_arch_dir_name());

    let mut candidates = vec![platform_dir.join(exe_name)];

    if cfg!(target_os = "windows") {
        candidates.push(platform_dir.join("vgmstream-win64").join(exe_name));
    }

    candidates.push(root.join(exe_name));
    if cfg!(target_os = "windows") {
        candidates.push(root.join("vgmstream-win64").join(exe_name));
    }

    candidates
}

pub fn find_vgmstream_cli() -> Option<PathBuf> {
    vgmstream_cli_candidates()
        .into_iter()
        .find(|path| path.is_file())
}

fn vgmstream_cli_exe_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "vgmstream-cli.exe"
    } else {
        "vgmstream-cli"
    }
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
