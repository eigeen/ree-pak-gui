use std::{env, fs, path::Path};

use self_update::Extract;

#[test]
fn packaged_release_archive_extracts_expected_binary() -> Result<(), Box<dyn std::error::Error>> {
    let Some(archive_path) = env::var_os("REE_PAK_RELEASE_ARCHIVE_PATH") else {
        return Ok(());
    };
    let bin_path = env::var("REE_PAK_RELEASE_BIN_PATH")?;
    let output_dir = tempfile::tempdir()?;

    Extract::from_source(Path::new(&archive_path)).extract_file(output_dir.path(), &bin_path)?;
    assert_extracted_binary_is_present(output_dir.path(), &bin_path)
}

fn assert_extracted_binary_is_present(
    output_dir: &Path,
    bin_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let extracted_path = output_dir.join(bin_path);
    let metadata = fs::metadata(&extracted_path)?;

    if !metadata.is_file() || metadata.len() == 0 {
        return Err(format!("extracted binary is invalid: {}", extracted_path.display()).into());
    }

    Ok(())
}
