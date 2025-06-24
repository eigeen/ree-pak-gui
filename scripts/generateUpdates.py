import os
import subprocess
import json
import hashlib
import zipfile
from packaging import version


def get_current_version():
    """从 Cargo.toml 获取当前版本"""
    with open(os.path.join("src-tauri", "Cargo.toml"), "r") as f:
        for line in f:
            if line.startswith("version"):
                return line.split("=")[1].strip().strip('"')
    raise ValueError("Failed to get version from Cargo.toml")


def get_commit_info():
    """获取提交时间和简短hash"""
    commit_time = (
        subprocess.check_output(["git", "log", "-1", "--format=%cI"]).decode().strip()
    )
    commit_hash_short = (
        subprocess.check_output(["git", "log", "-1", "--format=%h"]).decode().strip()
    )
    return commit_time, commit_hash_short


def create_zip_file(file_path, zip_name):
    """创建指定名称的zip压缩包，压缩包内的文件名与压缩包名（去掉 .zip 后缀）一致"""
    inner_name = os.path.splitext(os.path.basename(zip_name))[0]  # 去掉 .zip 后缀
    with zipfile.ZipFile(zip_name, "w", compression=zipfile.ZIP_DEFLATED) as zipf:
        zipf.write(file_path, arcname=inner_name)
    return zip_name


def get_file_info(file_path):
    """获取文件大小和哈希"""
    file_size = os.path.getsize(file_path)
    file_hash = hashlib.sha256(open(file_path, "rb").read()).hexdigest()
    return file_size, file_hash


def load_update_db():
    """加载 update_db.json"""
    script_path = os.path.dirname(os.path.realpath(__file__))
    db_path = os.path.join(script_path, "update_db.json")
    if os.path.exists(db_path):
        with open(db_path, "r") as f:
            return json.load(f)
    return []


def save_update_db(data):
    """保存 update_db.json"""
    script_path = os.path.dirname(os.path.realpath(__file__))
    db_path = os.path.join(script_path, "update_db.json")
    with open(db_path, "w") as f:
        json.dump(data, f, indent=4)


def generate_update_info():
    """生成更新信息"""
    current_version = get_current_version()
    commit_time, commit_hash_short = get_commit_info()

    # 获取可执行文件路径
    exe_path = os.path.join("..", "target", "release", "ree-pak-rs.exe")

    # 创建zip压缩包
    zip_name = f"ree-pak-gui_{current_version}_windows_x86_64_release_{commit_hash_short}.exe.zip"
    zip_path = os.path.join(os.path.dirname(exe_path), zip_name)
    create_zip_file(exe_path, zip_path)
    zip_size, zip_hash = get_file_info(zip_path)

    # 构建新版本信息
    new_version = {
        "version": current_version,
        "channel": "release",
        "pub_time": commit_time,
        "files": [
            {
                "name": zip_name,
                "size": zip_size,
                "sha256": zip_hash,
                "urls": [
                    f"https://github.com/eigeen/ree-pak-gui/releases/download/v{current_version}/{zip_name}",
                    f"https://os1.eigeen.com/ree-pak-gui/{zip_name}",
                ],
            }
        ],
    }

    # 加载历史版本
    update_db = load_update_db()

    # 检查是否需要更新
    should_update = True
    for idx, entry in enumerate(update_db):
        if version.parse(entry["version"]) == version.parse(current_version):
            # 版本相同，更新日期
            update_db[idx] = new_version
            should_update = False
            break
        elif version.parse(entry["version"]) > version.parse(current_version):
            # 历史版本更高，不更新
            should_update = False
            break

    if should_update:
        update_db.insert(0, new_version)

    # 保存更新后的数据库
    save_update_db(update_db)

    # 生成 update.json (包含所有版本)
    update_json = {"versions": update_db}
    with open(os.path.join("scripts", "update.json"), "w") as f:
        json.dump(update_json, f, indent=4)


if __name__ == "__main__":
    # switch work dir
    script_dir = os.path.dirname(os.path.realpath(__file__))
    os.chdir(os.path.join(script_dir, ".."))

    generate_update_info()
