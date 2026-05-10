use std::path::Path;

use crate::preview::PreviewMaterial;
use crate::read::{ParseError, ParseResult, Reader, file_version_from_path, read_file};

const MDF_MAGIC: u32 = 0x0046_444d;
const ALBEDO_TEXTURE_TYPES: &[&str] = &[
    "BaseDielectricMap",
    "ALBD",
    "ALBDmap",
    "BackMap",
    "BaseMetalMap",
    "BaseDielectricMapBase",
    "BaseAlphaMap",
    "BaseShiftMap",
];
const NORMAL_TEXTURE_TYPES: &[&str] = &["NormalRoughnessMap", "NormalRoughnessCavityMap"];

#[derive(Debug, Clone)]
pub struct MdfFile {
    pub version: u16,
    pub file_version: u32,
    pub material_flags: u64,
    pub materials: Vec<MaterialData>,
}

impl MdfFile {
    pub fn read_path(path: impl AsRef<Path>) -> ParseResult<Self> {
        let file_version = file_version_from_path(&path, ".mdf2")?;
        let data = read_file(path)?;
        Self::read_bytes(&data, file_version)
    }

    pub fn read_bytes(data: &[u8], file_version: u32) -> ParseResult<Self> {
        let mut reader = Reader::new(data);
        let magic = reader.read_u32()?;
        if magic != MDF_MAGIC {
            return Err(ParseError::InvalidMagic {
                expected: "MDF",
                actual: magic,
            });
        }

        let version = reader.read_u16()?;
        let material_count = usize::from(reader.read_u16()?);
        let material_flags = reader.read_u64()?;

        let mut materials = Vec::with_capacity(material_count);
        for _ in 0..material_count {
            materials.push(MaterialData {
                header: MaterialHeader::read(&mut reader, file_version)?,
                textures: Vec::new(),
            });
        }

        for material in &mut materials {
            if material.header.tex_header_offset == 0 {
                continue;
            }
            let mut tex_reader = reader.fork_at(material.header.tex_header_offset)?;
            for _ in 0..material.header.tex_count.max(0) {
                material
                    .textures
                    .push(TextureBinding::read(&mut tex_reader, file_version)?);
            }
        }

        Ok(Self {
            version,
            file_version,
            material_flags,
            materials,
        })
    }

    pub fn preview_materials_for(&self, material_names: &[String]) -> Vec<PreviewMaterial> {
        material_names
            .iter()
            .map(|name| {
                let material = self
                    .materials
                    .iter()
                    .find(|material| material.name() == name);
                PreviewMaterial {
                    name: name.clone(),
                    albedo_texture_path: material
                        .and_then(|material| material.first_texture_path(ALBEDO_TEXTURE_TYPES))
                        .or_else(|| material.and_then(MaterialData::fallback_albedo_path)),
                    normal_texture_path: material
                        .and_then(|material| material.first_texture_path(NORMAL_TEXTURE_TYPES)),
                }
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct MaterialData {
    pub header: MaterialHeader,
    pub textures: Vec<TextureBinding>,
}

impl MaterialData {
    pub fn name(&self) -> &str {
        &self.header.name
    }

    fn first_texture_path(&self, texture_types: &[&str]) -> Option<String> {
        self.textures
            .iter()
            .find(|texture| {
                texture_types.contains(&texture.texture_type.as_str())
                    && !is_null_texture_path(&texture.texture_path)
            })
            .map(|texture| texture.texture_path.clone())
    }

    fn fallback_albedo_path(&self) -> Option<String> {
        self.textures
            .iter()
            .find(|texture| {
                !is_null_texture_path(&texture.texture_path)
                    && texture.texture_path.to_ascii_lowercase().contains("_alb")
            })
            .map(|texture| texture.texture_path.clone())
    }
}

#[derive(Debug, Clone)]
pub struct MaterialHeader {
    pub name: String,
    pub name_hash: u32,
    pub params_size: i32,
    pub param_count: i32,
    pub tex_count: i32,
    pub shader_type: i32,
    pub alpha_flags: u32,
    pub param_header_offset: u64,
    pub tex_header_offset: u64,
    pub params_offset: u64,
    pub mmtr_path: String,
}

fn is_null_texture_path(path: &str) -> bool {
    path.is_empty() || path.to_ascii_lowercase().contains("/null")
}

impl MaterialHeader {
    fn read(reader: &mut Reader<'_>, file_version: u32) -> ParseResult<Self> {
        let name_offset = reader.read_u64()?;
        let name = reader.read_utf16_cstr_at(name_offset)?;
        let name_hash = reader.read_u32()?;
        if file_version == 6 {
            reader.skip(8)?;
        }
        let params_size = reader.read_i32()?;
        let param_count = reader.read_i32()?;
        let tex_count = reader.read_i32()?;
        if file_version >= 19 {
            let gpbf_name_count = reader.read_i32()?;
            let gpbf_data_count = reader.read_i32()?;
            if gpbf_name_count != gpbf_data_count {
                return Err(ParseError::Unsupported("MDF GPBF count mismatch"));
            }
        }
        let shader_type = reader.read_i32()?;
        if file_version >= 32 {
            reader.skip(4)?;
        }
        let alpha_flags = reader.read_u32()?;
        if file_version >= 32 {
            reader.skip(8)?;
        }
        if file_version >= 51 {
            reader.skip(8)?;
        }
        let param_header_offset = reader.read_u64()?;
        let tex_header_offset = reader.read_u64()?;
        if file_version >= 19 {
            reader.skip(8)?;
        }
        let params_offset = reader.read_u64()?;
        let mmtr_path = reader.read_offset_utf16_string()?;
        if file_version >= 32 {
            reader.skip(8)?;
        }

        Ok(Self {
            name,
            name_hash,
            params_size,
            param_count,
            tex_count,
            shader_type,
            alpha_flags,
            param_header_offset,
            tex_header_offset,
            params_offset,
            mmtr_path,
        })
    }
}

#[derive(Debug, Clone)]
pub struct TextureBinding {
    pub texture_type: String,
    pub unicode_hash: u32,
    pub ascii_hash: u32,
    pub texture_path: String,
}

impl TextureBinding {
    fn read(reader: &mut Reader<'_>, file_version: u32) -> ParseResult<Self> {
        let texture_type = reader.read_offset_utf16_string()?;
        let unicode_hash = reader.read_u32()?;
        let ascii_hash = reader.read_u32()?;
        let texture_path = reader.read_offset_utf16_string()?;
        if file_version >= 13 {
            reader.skip(8)?;
        }
        Ok(Self {
            texture_type,
            unicode_hash,
            ascii_hash,
            texture_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MDF: &str =
        "test_files/mhs3/natives/STM/Art/Model/character/ch00/000/0000/ch00_000_0000.mdf2.49";

    #[test]
    fn reads_mhs3_mdf_materials_and_textures() {
        let mdf = MdfFile::read_path(SAMPLE_MDF).expect("mdf should parse");

        assert_eq!(mdf.version, 1);
        assert_eq!(mdf.file_version, 49);
        assert_eq!(mdf.materials.len(), 11);
        assert!(mdf.materials.iter().any(|mat| !mat.name().is_empty()));
        assert!(
            mdf.materials
                .iter()
                .flat_map(|mat| &mat.textures)
                .any(|tex| !tex.texture_path.is_empty())
        );

        let material_names = mdf
            .materials
            .iter()
            .map(|material| material.name().to_string())
            .collect::<Vec<_>>();
        let preview_materials = mdf.preview_materials_for(&material_names);
        assert_eq!(preview_materials.len(), material_names.len());
        assert!(
            preview_materials
                .iter()
                .any(|material| material.albedo_texture_path.is_some())
        );
    }
}
