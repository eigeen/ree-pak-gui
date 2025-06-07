use std::process::Command;

fn main() {
    let commit_time = get_commit_rfc3339().unwrap();
    println!("cargo:rustc-env=GIT_COMMIT_TIME_RFC3339={}", commit_time);

    let commit_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_err(|e| format!("Git 执行失败: {}", e))
        .and_then(|output| {
            if !output.status.success() {
                Err(format!("Git 命令失败: {}", output.status))
            } else {
                String::from_utf8(output.stdout)
                    .map_err(|e| format!("git output is not utf8: {}", e))
                    .map(|s| s.trim().to_string())
            }
        })
        .unwrap();
    println!("cargo:rustc-env=GIT_COMMIT_HASH={}", commit_hash);

    tauri_build::build()
}

fn get_commit_rfc3339() -> Result<String, String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cI"])
        .output()
        .map_err(|e| format!("Git 执行失败: {}", e))?;

    if !output.status.success() {
        return Err(format!("Git 命令失败: {}", output.status));
    }

    let time_str = String::from_utf8(output.stdout)
        .map_err(|e| format!("git output is not utf8: {}", e))?
        .trim()
        .to_string();

    Ok(time_str)
}
