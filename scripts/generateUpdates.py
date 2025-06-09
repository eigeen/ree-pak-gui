import os
import subprocess
import shutil
import json
import re
import hashlib
import zipfile

# Get .py file path
script_path = os.path.dirname(os.path.realpath(__file__))
working_dir = os.path.join(script_path, "..")
# Move to working directory
os.chdir(working_dir)

# Get version from src-tauri/Cargo.toml
version = None
with open(os.path.join(working_dir, "src-tauri", "Cargo.toml"), "r") as f:
    for line in f:
        if line.startswith("version"):
            version = line.split("=")[1].strip().strip('"')
            break
if not version:
    print("Failed to get version from Cargo.toml")
    exit(1)

# Get latest commit time RFC3339
commit_time = (
    subprocess.check_output(["git", "log", "-1", "--format=%cI"]).decode().strip()
)
commit_hash_short = (
    subprocess.check_output(["git", "log", "-1", "--format=%h"]).decode().strip()
)

# Get latest tagged commit time RFC3339
tag_time = (
    subprocess.check_output(["git", "log", "-1", "--format=%cI", "--tags"])
    .decode()
    .strip()
)

# Get file hash and size
file_path = os.path.join(working_dir, "..", "target", "release", "ree-pak-rs.exe")
file_size = os.path.getsize(file_path)
file_hash = hashlib.sha256(open(file_path, "rb").read()).hexdigest()

# # upx compress for nightly channel
# upx_file_path = file_path.replace(".exe", ".upx.exe")
# shutil.copyfile(file_path, upx_file_path)
# subprocess.check_call(["upx", "-9", upx_file_path])

# upx_file_size = os.path.getsize(upx_file_path)
# upx_file_hash = hashlib.sha256(open(upx_file_path, "rb").read()).hexdigest()

# zip compress
zip_file_path = file_path.replace(".exe", ".zip")
with zipfile.ZipFile(zip_file_path, "w", compression=zipfile.ZIP_DEFLATED) as zipf:
    zipf.write(file_path, arcname=os.path.basename(file_path))

zip_file_size = os.path.getsize(zip_file_path)
zip_file_hash = hashlib.sha256(open(zip_file_path, "rb").read()).hexdigest()

nightly_file_name = (
    f"ree-pak-gui_{version}_windows_x86_64_nightly_{commit_hash_short}.zip"
)

update_versions = {
    "versions": [
        {
            "version": version,
            "channel": "nightly",
            "pub_time": commit_time,
            "files": [
                {
                    "name": nightly_file_name,
                    "size": zip_file_size,
                    "sha256": zip_file_hash,
                    "urls": [
                        f"https://os1.eigeen.com/ree-pak-gui/{nightly_file_name}",
                    ],
                }
            ],
        }
    ]
}

release_update_version = None
# Generate release channel cache
if tag_time == commit_time:
    with open(os.path.join(working_dir, "scripts", "update-release.json"), "w") as f:
        release_file_name = f"ree-pak-gui_{version}_windows_x86_64.exe"
        release_update_version = {
            "version": version,
            "channel": "release",
            "pub_time": commit_time,
            "files": [
                {
                    "name": release_file_name,
                    "size": file_size,
                    "sha256": file_hash,
                    "urls": [
                        f"https://github.com/eigeen/ree-pak-gui/releases/download/v{version}/{release_file_name}",
                        f"https://os1.eigeen.com/ree-pak-gui/{release_file_name}",
                    ],
                }
            ],
        }
        json.dump(
            release_update_version,
            f,
            indent=4,
        )
else:
    # Load release channel cache
    if os.path.exists(os.path.join(working_dir, "scripts", "update-release.json")):
        with open(
            os.path.join(working_dir, "scripts", "update-release.json"), "r"
        ) as f:
            release_update_version = json.load(f)

# Merge release and nightly
if release_update_version:
    update_versions["versions"].append(release_update_version)

# Write update.json
with open(os.path.join(working_dir, "scripts", "update.json"), "w") as f:
    json.dump(
        update_versions,
        f,
        indent=4,
    )

# Copy files to dist directory
target_dir = os.path.join(working_dir, "..", "target", "release")
if os.path.exists(target_dir):
    for version in update_versions["versions"]:
        f_from = zip_file_path if version["channel"] == "nightly" else file_path
        for file in version["files"]:
            shutil.copyfile(
                f_from,
                os.path.join(target_dir, file["name"]),
            )
