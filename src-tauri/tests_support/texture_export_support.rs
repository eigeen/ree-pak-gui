use std::{
    env,
    fs,
    io::{Cursor, Read, Write},
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use image::{DynamicImage, ImageFormat};
use re_tex::tex::Tex;
use ree_pak_core::{pakfile::PakFile, utf16_hash::Utf16HashExt, write::{FileOptions, PakWriter}};
use walkdir::WalkDir;

const FIXTURE_ENV: &str = "REE_PAK_BENCH_TEX";
const DEFAULT_FIXTURE_NAMES: &[&str] = &[
    "ch04_000_0000_1002_MB.tex.241106027",
    "ch04_000_0000_1001_ALBD.tex.241106027",
];

#[derive(Debug, Clone)]
pub struct TextureFixture {
    pub name: String,
    pub entry_path: String,
    pub hash: u64,
    pub tex_len: usize,
    pub pak_len: usize,
    tex_bytes: Vec<u8>,
    pak_bytes: Vec<u8>,
}

impl TextureFixture {
    fn from_tex_path(path: &Path) -> Result<Self> {
        let tex_bytes = fs::read(path)
            .with_context(|| format!("failed to read texture fixture: {}", path.display()))?;
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_owned)
            .with_context(|| format!("fixture path has no valid file name: {}", path.display()))?;
        let entry_path = format!("benchmark/{file_name}");
        let hash = entry_path.hash_mixed();
        let pak_bytes = build_pak_bytes(&entry_path, &tex_bytes)?;

        Ok(Self {
            name: file_name,
            entry_path,
            hash,
            tex_len: tex_bytes.len(),
            pak_len: pak_bytes.len(),
            tex_bytes,
            pak_bytes,
        })
    }

    pub fn tex_bytes(&self) -> &[u8] {
        &self.tex_bytes
    }

    pub fn pak_bytes(&self) -> &[u8] {
        &self.pak_bytes
    }
}

pub fn load_texture_fixtures() -> Result<Vec<TextureFixture>> {
    let mut fixture_paths = load_fixture_paths_from_env()?;
    if fixture_paths.is_empty() {
        fixture_paths = discover_default_fixture_paths()?;
    }

    if fixture_paths.is_empty() {
        bail!(
            "no texture fixtures found; set {FIXTURE_ENV} to one or more .tex paths separated by the platform path separator"
        );
    }

    fixture_paths.sort_by(|left, right| {
        fixture_rank(left)
            .cmp(&fixture_rank(right))
            .then_with(|| left.cmp(right))
    });
    fixture_paths.dedup();

    fixture_paths
        .into_iter()
        .map(|path| TextureFixture::from_tex_path(&path))
        .collect()
}

pub fn read_entry_from_memory_pak(fixture: &TextureFixture) -> Result<Vec<u8>> {
    let pak = PakFile::from_reader(Cursor::new(fixture.pak_bytes()))?;
    let entry = pak
        .metadata()
        .entries()
        .iter()
        .find(|entry| entry.hash() == fixture.hash)
        .with_context(|| format!("fixture entry not found in pak: {}", fixture.entry_path))?;

    let mut reader = pak.open_entry(entry)?;
    let mut bytes = Vec::with_capacity(entry.uncompressed_size() as usize);
    reader
        .read_to_end(&mut bytes)
        .context("failed to read entry bytes from in-memory pak")?;
    Ok(bytes)
}

pub fn export_dds_from_memory_pak(fixture: &TextureFixture) -> Result<Vec<u8>> {
    let tex_bytes = read_entry_from_memory_pak(fixture)?;
    export_dds_from_tex_bytes(&tex_bytes)
}

pub fn export_png_from_memory_pak(fixture: &TextureFixture) -> Result<Vec<u8>> {
    let tex_bytes = read_entry_from_memory_pak(fixture)?;
    export_png_from_tex_bytes(&tex_bytes)
}

pub fn export_dds_from_tex_bytes(tex_bytes: &[u8]) -> Result<Vec<u8>> {
    let tex = parse_tex(tex_bytes)?;
    let dds = tex
        .to_dds(tex.header.mipmap_count as usize)
        .context("failed to convert tex to dds")?;

    let mut output = Cursor::new(Vec::new());
    dds.write(&mut output)
        .context("failed to write dds bytes to memory")?;
    Ok(output.into_inner())
}

pub fn export_png_from_tex_bytes(tex_bytes: &[u8]) -> Result<Vec<u8>> {
    let tex = parse_tex(tex_bytes)?;
    let image = tex.to_rgba_image(0).context("failed to decode tex to rgba")?;

    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut output, ImageFormat::Png)
        .context("failed to write png bytes to memory")?;
    Ok(output.into_inner())
}

fn parse_tex(tex_bytes: &[u8]) -> Result<Tex> {
    let mut reader = Cursor::new(tex_bytes);
    Tex::from_reader(&mut reader).context("failed to parse tex bytes")
}

fn build_pak_bytes(entry_path: &str, tex_bytes: &[u8]) -> Result<Vec<u8>> {
    let mut pak_bytes = Vec::new();
    {
        let cursor = Cursor::new(&mut pak_bytes);
        let mut writer = PakWriter::new(cursor, 1);
        writer
            .start_file(entry_path, FileOptions::default())
            .context("failed to start pak entry")?;
        writer
            .write_all(tex_bytes)
            .context("failed to write tex bytes into pak")?;
        writer.finish().context("failed to finalize pak bytes")?;
    }
    Ok(pak_bytes)
}

fn load_fixture_paths_from_env() -> Result<Vec<PathBuf>> {
    let Some(raw_paths) = env::var_os(FIXTURE_ENV) else {
        return Ok(Vec::new());
    };

    env::split_paths(&raw_paths)
        .map(|path| {
            if path.exists() {
                Ok(path)
            } else {
                bail!("fixture path from {FIXTURE_ENV} does not exist: {}", path.display())
            }
        })
        .collect()
}

fn discover_default_fixture_paths() -> Result<Vec<PathBuf>> {
    let checkout_root = cargo_home_dir().join("git").join("checkouts");
    if !checkout_root.exists() {
        return Ok(Vec::new());
    }

    let mut matches = Vec::new();
    for file_name in DEFAULT_FIXTURE_NAMES {
        let Some(path) = WalkDir::new(&checkout_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .find(|entry| entry.file_type().is_file() && entry.file_name() == *file_name)
            .map(|entry| entry.into_path())
        else {
            continue;
        };
        matches.push(path);
    }

    Ok(matches)
}

fn cargo_home_dir() -> PathBuf {
    if let Some(cargo_home) = env::var_os("CARGO_HOME") {
        return PathBuf::from(cargo_home);
    }

    let home = env::var_os("USERPROFILE")
        .or_else(|| env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".cargo")
}

fn fixture_rank(path: &Path) -> usize {
    let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    DEFAULT_FIXTURE_NAMES
        .iter()
        .position(|expected| *expected == file_name)
        .unwrap_or(DEFAULT_FIXTURE_NAMES.len())
}
