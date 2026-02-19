import hashlib
import zipfile
import json
import os
import shutil
from typing import Dict, Any


def parse_metadata(file_path: str) -> Dict[str, Any]:
    """Parse metadata from file (lines starting with #!)"""
    metadata = {}
    try:
        with open(file_path, "r", encoding="utf-8") as f:
            for line in f:
                line = line.strip()
                if not line.startswith("#!"):
                    break

                meta_line = line[2:].strip()
                separator_index = meta_line.find(":")
                if separator_index == -1:
                    continue

                key = meta_line[:separator_index].strip()
                value = meta_line[separator_index + 1 :].strip()

                if key.startswith("@"):
                    clean_key = key[1:]
                    metadata[clean_key] = (
                        value.split(",") if clean_key == "tags" else value
                    )
    except:
        pass
    return metadata


def main():
    # Get .py file path
    script_path = os.path.dirname(os.path.realpath(__file__))
    working_dir = os.path.join(script_path, "..")
    # Move to working directory
    os.chdir(working_dir)

    filelist_input_dir = "assets/filelist"
    filelist_manifest_path = "scripts/filelist_manifest.json"
    filelist_output_dir = "scripts/filelist"

    # Create output directory
    if os.path.exists(filelist_output_dir):
        shutil.rmtree(filelist_output_dir)
    os.makedirs(filelist_output_dir)

    # Zip every files
    for root, dirs, files in os.walk(filelist_input_dir):
        for file in files:
            file_path = os.path.join(root, file)
            with zipfile.ZipFile(
                file=os.path.join(filelist_output_dir, file + ".zip"),
                mode="w",
                compression=zipfile.ZIP_DEFLATED,
            ) as zipf:
                zipf.write(file_path, os.path.basename(file_path))

    # Generate manifest
    manifest = {
        "base_urls": [
            "https://raw.githubusercontent.com/eigeen/ree-pak-gui-update/refs/heads/main/filelist",
            "https://os1.eigeen.com/ree-pak-gui/filelist",
        ],
        "files": [],
    }

    for root, dirs, files in os.walk(filelist_output_dir):
        for file in files:
            file_path = os.path.join(root, file)
            with open(file_path, "rb") as f:
                file_hash = hashlib.sha256(f.read()).hexdigest()

            # Get original file path to parse metadata
            orig_file = os.path.join(filelist_input_dir, file[:-4])  # remove .zip
            metadata = parse_metadata(orig_file)

            manifest["files"].append(
                {
                    "file_name": os.path.basename(file_path),
                    "tags": metadata.get("tags", []),
                    "update_time": metadata["update_time"],  # must have update_time
                    "description": metadata.get("description", ""),
                    "size": os.path.getsize(file_path),
                    "sha256": file_hash,
                }
            )

    with open(filelist_manifest_path, "w", encoding="utf-8") as f:
        json.dump(manifest, f, indent=2, ensure_ascii=False)


if __name__ == "__main__":
    main()
