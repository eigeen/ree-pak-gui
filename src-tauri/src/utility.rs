use std::{fs::File, io::BufReader, path::Path};

use zip::ZipArchive;

use crate::error::Result;

#[cfg(target_os = "windows")]
pub fn message_box_error(message: &str) {
    use windows::{
        Win32::UI::WindowsAndMessaging::{MB_ICONERROR, MessageBoxW},
        core::{PCWSTR, w},
    };

    let msg = to_utf16_with_nul(message);
    unsafe {
        MessageBoxW(None, PCWSTR(msg.as_ptr()), w!("Ree Pak GUI Error"), MB_ICONERROR);
    }
}

#[cfg(target_os = "windows")]
fn to_utf16_with_nul(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect::<Vec<_>>()
}

/// Extract all files from a zip archive to a directory.
///
/// Returns a list of extracted file paths.
pub fn zip_extract_all(input: impl AsRef<Path>, output_root: impl AsRef<Path>) -> Result<Vec<String>> {
    let input = input.as_ref();
    let output_root = output_root.as_ref();

    let reader = BufReader::new(File::open(input)?);
    let mut zip = ZipArchive::new(reader)?;

    if !output_root.exists() {
        std::fs::create_dir_all(output_root)?;
    }

    let mut extracted_files = vec![];
    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        if let Some(p) = file.enclosed_name() {
            let outpath = output_root.join(p);
            if let Some(p) = outpath.parent()
                && !p.exists()
            {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
            extracted_files.push(outpath.to_string_lossy().to_string());
        }
    }

    Ok(extracted_files)
}
