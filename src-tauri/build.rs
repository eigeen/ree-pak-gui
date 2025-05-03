use std::process::Command;

fn main() {
    let commit_time = get_commit_rfc3339().unwrap();
    println!("cargo:rustc-env=GIT_COMMIT_TIME_RFC3339={}", commit_time);

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
