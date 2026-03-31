#[path = "../tests_support/texture_export_support.rs"]
mod texture_export_support;

use anyhow::Result;
use texture_export_support::{
    export_dds_from_memory_pak, export_png_from_memory_pak, load_texture_fixtures,
    read_entry_from_memory_pak,
};

#[test]
fn memory_pak_reader_returns_original_tex_bytes() -> Result<()> {
    for fixture in load_texture_fixtures()? {
        let tex_bytes = read_entry_from_memory_pak(&fixture)?;
        assert_eq!(tex_bytes, fixture.tex_bytes());
    }

    Ok(())
}

#[test]
fn dds_export_stays_in_memory() -> Result<()> {
    for fixture in load_texture_fixtures()? {
        let dds_bytes = export_dds_from_memory_pak(&fixture)?;
        assert!(dds_bytes.starts_with(b"DDS "));
    }

    Ok(())
}

#[test]
fn png_export_stays_in_memory() -> Result<()> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";

    for fixture in load_texture_fixtures()? {
        let png_bytes = export_png_from_memory_pak(&fixture)?;
        assert!(png_bytes.starts_with(PNG_SIGNATURE));
    }

    Ok(())
}
