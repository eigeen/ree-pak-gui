use std::path::Path;

use crate::{
    preview::{PreviewBounds, PreviewMaterial, PreviewModel, PreviewSubmesh},
    read::{ParseError, ParseResult, Reader, f16_to_f32, file_version_from_path, read_file},
};

const MESH_MAGIC: u32 = 0x4853_454d;
const MPLY_MAGIC: u32 = 0x594c_504d;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MeshSerializerVersion {
    Unknown,
    Re7,
    Dmc5,
    ReRt,
    Re8,
    Re4,
    Sf6,
    Dd2Old,
    Dd2,
    Onimusha,
    MhWilds,
    Pragmata,
    Re9,
}

impl MeshSerializerVersion {
    pub fn resolve(internal_version: u32, file_version: u32) -> Self {
        match (internal_version, file_version) {
            (386270720, 1808282334 | 1808312334 | 1902042334) => Self::Dmc5,
            (21041600, 2109108288 | 220128762) => Self::ReRt,
            (21061800 | 21091000, 2109148288) => Self::ReRt,
            (2020091500, 2101050001) => Self::Re8,
            (220822879, 221108797) => Self::Re4,
            (220705151, 230110883) => Self::Sf6,
            (230517984, 231011879) => Self::Dd2Old,
            (230517984, 240423143) => Self::Dd2,
            (230727984, 240306278) => Self::Dd2Old,
            (240704828, 240827123) => Self::Onimusha,
            (240704828, 241111606) => Self::MhWilds,
            (250203152, 250604100) => Self::Pragmata,
            (250707828, 250925211 | 251121828) => Self::Pragmata,
            (250904410, 250925211) => Self::Re9,
            _ if file_version == 250604100 => Self::Pragmata,
            _ if file_version == 241111606 => Self::MhWilds,
            _ if file_version == 250925211 => Self::Re9,
            _ => Self::MhWilds,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshHeader {
    pub version: u32,
    pub file_size: u32,
    pub lod_hash: u32,
    pub flags: u16,
    pub name_count: u16,
    pub lods_offset: u64,
    pub shadow_lods_offset: u64,
    pub occluder_mesh_offset: u64,
    pub bones_offset: u64,
    pub normal_recalc_offset: u64,
    pub blend_shape_headers_offset: u64,
    pub bounds_offset: u64,
    pub mesh_offset: u64,
    pub floats_offset: u64,
    pub material_indices_offset: u64,
    pub bone_indices_offset: u64,
    pub blend_shape_indices_offset: u64,
    pub name_offsets_offset: u64,
    pub streaming_info_offset: u64,
    pub vertices_offset: u64,
    pub serializer_version: MeshSerializerVersion,
}

impl MeshHeader {
    pub fn buffer_count(&self) -> u16 {
        (self.flags >> 9) & 0x7
    }

    fn read(reader: &mut Reader<'_>, file_version: u32) -> ParseResult<Self> {
        let magic = reader.read_u32()?;
        if magic == MPLY_MAGIC {
            return Err(ParseError::Unsupported(
                "MPLY meshlet meshes are detected but not converted yet",
            ));
        }
        if magic != MESH_MAGIC {
            return Err(ParseError::InvalidMagic {
                expected: "MESH",
                actual: magic,
            });
        }

        let version = reader.read_u32()?;
        let file_size = reader.read_u32()?;
        let lod_hash = reader.read_u32()?;
        let serializer_version = MeshSerializerVersion::resolve(version, file_version);

        if serializer_version >= MeshSerializerVersion::Onimusha {
            let _wilds_unkn1 = reader.read_i32()?;
            let name_count = reader.read_u16()?;
            let flags = reader.read_u16()?;
            let _ukn_count = reader.read_i16()?;
            let _wilds_unkn2 = reader.read_i32()?;
            let _wilds_unkn3 = reader.read_i32()?;
            let _wilds_unkn4 = reader.read_i32()?;
            let _wilds_unkn5 = reader.read_i16()?;
            let vertices_offset = reader.read_u64()?;
            let lods_offset = reader.read_u64()?;
            let shadow_lods_offset = reader.read_u64()?;
            let occluder_mesh_offset = reader.read_u64()?;
            let normal_recalc_offset = reader.read_u64()?;
            let blend_shape_headers_offset = reader.read_u64()?;
            let mesh_offset = reader.read_u64()?;
            let _sf6_unkn1 = reader.read_u64()?;
            let floats_offset = reader.read_u64()?;
            let bounds_offset = reader.read_u64()?;
            let bones_offset = reader.read_u64()?;
            let material_indices_offset = reader.read_u64()?;
            let bone_indices_offset = reader.read_u64()?;
            let blend_shape_indices_offset = reader.read_u64()?;
            let name_offsets_offset = reader.read_u64()?;
            let streaming_info_offset = reader.read_u64()?;
            let _sf6_unkn4 = reader.read_u64()?;

            return Ok(Self {
                version,
                file_size,
                lod_hash,
                flags,
                name_count,
                lods_offset,
                shadow_lods_offset,
                occluder_mesh_offset,
                bones_offset,
                normal_recalc_offset,
                blend_shape_headers_offset,
                bounds_offset,
                mesh_offset,
                floats_offset,
                material_indices_offset,
                bone_indices_offset,
                blend_shape_indices_offset,
                name_offsets_offset,
                streaming_info_offset,
                vertices_offset,
                serializer_version,
            });
        }

        if serializer_version >= MeshSerializerVersion::Re4 {
            let flags = reader.read_u16()?;
            let _ukn_count = reader.read_i16()?;
            let name_count = reader.read_u16()?;
            let _ukn1 = reader.read_i16()?;
            let _ukn_offset = reader.read_u64()?;
            let lods_offset = reader.read_u64()?;
            let shadow_lods_offset = reader.read_u64()?;
            let occluder_mesh_offset = reader.read_u64()?;
            let normal_recalc_offset = reader.read_u64()?;
            let blend_shape_headers_offset = reader.read_u64()?;
            let mesh_offset = reader.read_u64()?;
            let _sf6_unkn1 = reader.read_u64()?;
            let floats_offset = reader.read_u64()?;
            let bounds_offset = reader.read_u64()?;
            let bones_offset = reader.read_u64()?;
            let material_indices_offset = reader.read_u64()?;
            let bone_indices_offset = reader.read_u64()?;
            let blend_shape_indices_offset = reader.read_u64()?;
            let (name_offsets_offset, _dd2_hash_offset, streaming_info_offset) =
                if serializer_version < MeshSerializerVersion::Dd2Old {
                    let streaming_info_offset = reader.read_u64()?;
                    let name_offsets_offset = reader.read_u64()?;
                    (name_offsets_offset, 0, streaming_info_offset)
                } else {
                    let name_offsets_offset = reader.read_u64()?;
                    let dd2_hash_offset = reader.read_u64()?;
                    let streaming_info_offset = reader.read_u64()?;
                    (name_offsets_offset, dd2_hash_offset, streaming_info_offset)
                };
            let vertices_offset = reader.read_u64()?;
            let _sdf_tex_path_offset = reader.read_u64()?;

            return Ok(Self {
                version,
                file_size,
                lod_hash,
                flags,
                name_count,
                lods_offset,
                shadow_lods_offset,
                occluder_mesh_offset,
                bones_offset,
                normal_recalc_offset,
                blend_shape_headers_offset,
                bounds_offset,
                mesh_offset,
                floats_offset,
                material_indices_offset,
                bone_indices_offset,
                blend_shape_indices_offset,
                name_offsets_offset,
                streaming_info_offset,
                vertices_offset,
                serializer_version,
            });
        }

        Err(ParseError::Unsupported(
            "pre-RE4 mesh headers are not implemented in this first parser pass",
        ))
    }
}

#[derive(Debug, Clone)]
pub struct MeshFile {
    pub header: MeshHeader,
    pub material_names: Vec<String>,
    pub mesh_buffer: Option<MeshBuffer>,
    pub streaming_info: Option<MeshStreamingInfo>,
    pub mesh_data: Option<MeshData>,
}

impl MeshFile {
    pub fn read_path(path: impl AsRef<Path>) -> ParseResult<Self> {
        let file_version = file_version_from_path(&path, ".mesh")?;
        let data = read_file(path)?;
        Self::read_bytes(&data, file_version)
    }

    pub fn read_bytes(data: &[u8], file_version: u32) -> ParseResult<Self> {
        let mut reader = Reader::new(data);
        let header = MeshHeader::read(&mut reader, file_version)?;

        let streaming_info = if header.streaming_info_offset > 0 {
            let mut stream_reader = reader.fork_at(header.streaming_info_offset)?;
            Some(MeshStreamingInfo::read(&mut stream_reader)?)
        } else {
            None
        };

        let mut mesh_buffer = if header.mesh_offset > 0 {
            let mut buffer_reader = reader.fork_at(header.mesh_offset)?;
            Some(MeshBuffer::read(
                &mut buffer_reader,
                header.serializer_version,
            )?)
        } else {
            None
        };

        let mesh_data = if header.lods_offset > 0 {
            let buffer = mesh_buffer
                .as_ref()
                .ok_or(ParseError::Unsupported("mesh LODs without a mesh buffer"))?;
            let mut mesh_reader = reader.fork_at(header.lods_offset)?;
            Some(MeshData::read(
                &mut mesh_reader,
                buffer,
                header.serializer_version,
            )?)
        } else {
            None
        };

        if let Some(buffer) = mesh_buffer.as_mut() {
            buffer.read_buffer_data(
                &reader,
                mesh_data.as_ref().is_some_and(|data| data.integer_faces),
                header.shadow_lods_offset > 0,
                header.occluder_mesh_offset > 0,
            )?;
        }

        let strings = read_strings(&reader, header.name_offsets_offset, header.name_count)?;
        let material_names = if header.material_indices_offset > 0 {
            let mut mat_reader = reader.fork_at(header.material_indices_offset)?;
            let mat_count = mesh_data
                .as_ref()
                .map(|data| data.material_count)
                .unwrap_or_default();
            let mut names = Vec::with_capacity(mat_count);
            for _ in 0..mat_count {
                let index = usize::from(mat_reader.read_u16()?);
                names.push(strings.get(index).cloned().unwrap_or_default());
            }
            names
        } else {
            Vec::new()
        };

        Ok(Self {
            header,
            material_names,
            mesh_buffer,
            streaming_info,
            mesh_data,
        })
    }

    pub fn to_preview_model(&self) -> ParseResult<PreviewModel> {
        let buffer = self
            .mesh_buffer
            .as_ref()
            .ok_or(ParseError::Unsupported("mesh has no vertex buffer"))?;
        let mesh_data = self
            .mesh_data
            .as_ref()
            .ok_or(ParseError::Unsupported("mesh has no LOD data"))?;
        let lod0 = mesh_data
            .lods
            .first()
            .ok_or(ParseError::Unsupported("mesh has no LOD0"))?;

        let mut meshes = Vec::new();
        for group in &lod0.mesh_groups {
            for (submesh_index, submesh) in group.submeshes.iter().enumerate() {
                if submesh.buffer_index != 0 {
                    continue;
                }
                meshes.push(buffer.preview_submesh(
                    submesh,
                    mesh_data.integer_faces,
                    group.group_id,
                    submesh_index,
                    self.material_names.get(usize::from(submesh.material_index)),
                )?);
            }
        }

        let materials = self
            .material_names
            .iter()
            .map(|name| PreviewMaterial {
                name: name.clone(),
                albedo_texture_path: None,
                normal_texture_path: None,
            })
            .collect();

        Ok(PreviewModel {
            meshes,
            materials,
            bounds: PreviewBounds {
                min: [
                    mesh_data.bounding_box_min[0],
                    mesh_data.bounding_box_min[1],
                    mesh_data.bounding_box_min[2],
                ],
                max: [
                    mesh_data.bounding_box_max[0],
                    mesh_data.bounding_box_max[1],
                    mesh_data.bounding_box_max[2],
                ],
                sphere_center: [
                    mesh_data.bounding_sphere[0],
                    mesh_data.bounding_sphere[1],
                    mesh_data.bounding_sphere[2],
                ],
                sphere_radius: mesh_data.bounding_sphere[3],
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct MeshStreamingInfo {
    pub entries: Vec<StreamingMeshEntry>,
}

#[derive(Debug, Clone, Copy)]
pub struct StreamingMeshEntry {
    pub start: u32,
    pub size: u32,
}

impl MeshStreamingInfo {
    fn read(reader: &mut Reader<'_>) -> ParseResult<Self> {
        let entry_count = reader.read_i32()?.max(0) as usize;
        reader.skip(4)?;
        let entry_offset = reader.read_u64()?;
        let mut entries = Vec::with_capacity(entry_count);
        if entry_count > 0 && entry_offset > 0 {
            let mut entry_reader = reader.fork_at(entry_offset)?;
            for _ in 0..entry_count {
                entries.push(StreamingMeshEntry {
                    start: entry_reader.read_u32()?,
                    size: entry_reader.read_u32()?,
                });
            }
        }
        Ok(Self { entries })
    }
}

#[derive(Debug, Clone)]
pub struct MeshBuffer {
    pub element_headers_offset: u64,
    pub vertex_buffer_offset: u64,
    pub total_buffer_size: i32,
    pub vertex_buffer_size: i32,
    pub face_buffer_total_size: i32,
    pub element_count: u16,
    pub total_element_count: u16,
    pub shadow_face_buffer_offset: i32,
    pub occ_face_buffer_offset: i32,
    pub blend_shape_offset: i32,
    pub headers: Vec<MeshBufferItemHeader>,
    pub additional_headers: Vec<Vec<MeshBufferItemHeader>>,
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uv0: Vec<[f32; 2]>,
    pub faces: Option<Vec<u16>>,
    pub integer_faces: Option<Vec<u32>>,
}

impl MeshBuffer {
    fn read(
        reader: &mut Reader<'_>,
        serializer_version: MeshSerializerVersion,
    ) -> ParseResult<Self> {
        let element_headers_offset = reader.read_u64()?;
        let vertex_buffer_offset = reader.read_u64()?;
        let _shapekey_weight_buffer_offset = if serializer_version >= MeshSerializerVersion::Re4 {
            reader.read_u64()?
        } else {
            0
        };
        let (total_buffer_size, vertex_buffer_size, face_buffer_total_size) =
            if serializer_version >= MeshSerializerVersion::Re4 {
                let total_buffer_size = reader.read_i32()?;
                let vertex_buffer_size = reader.read_i32()?;
                (
                    total_buffer_size,
                    vertex_buffer_size,
                    total_buffer_size - vertex_buffer_size,
                )
            } else {
                let _face_buffer_offset = reader.read_u64()?;
                if serializer_version == MeshSerializerVersion::ReRt {
                    reader.skip(8)?;
                }
                let vertex_buffer_size = reader.read_i32()?;
                let face_buffer_total_size = reader.read_i32()?;
                (
                    vertex_buffer_size + face_buffer_total_size,
                    vertex_buffer_size,
                    face_buffer_total_size,
                )
            };
        let element_count = reader.read_u16()?;
        let total_element_count = reader.read_u16()?;
        if serializer_version >= MeshSerializerVersion::Pragmata {
            reader.skip(16)?;
        }
        let mut shadow_face_buffer_offset = reader.read_i32()?;
        let mut occ_face_buffer_offset = reader.read_i32()?;
        if serializer_version < MeshSerializerVersion::Re4 {
            shadow_face_buffer_offset += vertex_buffer_size;
            occ_face_buffer_offset += vertex_buffer_size;
        }
        let blend_shape_offset = reader.read_i32()?;
        if serializer_version >= MeshSerializerVersion::Re4 {
            let _shapekey_weight_buffer_size = reader.read_i32()?;
            let _buffer_index = reader.read_i32()?;
            let _buffer_ukn1 = reader.read_i32()?;
            let _buffer_ukn2 = reader.read_i32()?;
        }

        let mut header_reader = reader.fork_at(element_headers_offset)?;
        let headers = read_buffer_headers(&mut header_reader, usize::from(element_count))?;

        let remaining = total_element_count.saturating_sub(element_count);
        let mut additional_headers = Vec::new();
        if remaining > 0 {
            let extra_headers = read_buffer_headers(&mut header_reader, usize::from(remaining))?;
            for header in extra_headers {
                if matches!(header.kind, VertexBufferType::Position) {
                    additional_headers.push(Vec::new());
                }
                if additional_headers.is_empty() {
                    additional_headers.push(Vec::new());
                }
                additional_headers.last_mut().unwrap().push(header);
            }
        }

        Ok(Self {
            element_headers_offset,
            vertex_buffer_offset,
            total_buffer_size,
            vertex_buffer_size,
            face_buffer_total_size,
            element_count,
            total_element_count,
            shadow_face_buffer_offset,
            occ_face_buffer_offset,
            blend_shape_offset,
            headers,
            additional_headers,
            positions: Vec::new(),
            normals: Vec::new(),
            uv0: Vec::new(),
            faces: None,
            integer_faces: None,
        })
    }

    fn read_buffer_data(
        &mut self,
        source: &Reader<'_>,
        integer_faces: bool,
        has_shadow: bool,
        has_occlusion: bool,
    ) -> ParseResult<()> {
        let vertex_count = self.vertex_count_from_headers(&self.headers)?;
        for header in &self.headers {
            let mut reader =
                source.fork_at(self.vertex_buffer_offset + u64::from(header.offset))?;
            match header.kind {
                VertexBufferType::Position => {
                    self.positions = (0..vertex_count)
                        .map(|_| reader.read_vec3())
                        .collect::<ParseResult<_>>()?;
                }
                VertexBufferType::NormalsTangents => {
                    self.normals = (0..vertex_count)
                        .map(|_| read_quantized_normal(&mut reader))
                        .collect::<ParseResult<_>>()?;
                }
                VertexBufferType::Uv0 => {
                    self.uv0 = (0..vertex_count)
                        .map(|_| read_half2(&mut reader))
                        .collect::<ParseResult<_>>()?;
                }
                _ => {
                    let bytes = usize::from(header.size) * vertex_count;
                    reader.skip(bytes)?;
                }
            }
        }

        let face_buffer_offset = self.vertex_buffer_offset + self.vertex_buffer_size as u64;
        let main_faces_size = if !has_shadow && !has_occlusion {
            if self.shadow_face_buffer_offset > self.vertex_buffer_size {
                self.shadow_face_buffer_offset - self.vertex_buffer_size
            } else {
                self.face_buffer_total_size
            }
        } else {
            let next_offset = if has_shadow {
                self.shadow_face_buffer_offset
            } else {
                self.occ_face_buffer_offset
            };
            next_offset - self.vertex_buffer_size
        }
        .max(0) as usize;

        let mut face_reader = source.fork_at(face_buffer_offset)?;
        if integer_faces {
            let count = main_faces_size / 4;
            self.integer_faces = Some(
                (0..count)
                    .map(|_| face_reader.read_u32())
                    .collect::<ParseResult<_>>()?,
            );
        } else {
            let count = main_faces_size / 2;
            self.faces = Some(
                (0..count)
                    .map(|_| face_reader.read_u16())
                    .collect::<ParseResult<_>>()?,
            );
        }

        Ok(())
    }

    fn vertex_count_from_headers(&self, headers: &[MeshBufferItemHeader]) -> ParseResult<usize> {
        if headers.len() >= 2 {
            let first = headers[0];
            let second = headers[1];
            return Ok(
                second.offset.saturating_sub(first.offset) as usize / usize::from(first.size)
            );
        }
        if let Some(position) = headers
            .iter()
            .find(|header| matches!(header.kind, VertexBufferType::Position))
        {
            return Ok(self.vertex_buffer_size.max(0) as usize / usize::from(position.size));
        }
        Err(ParseError::Unsupported(
            "mesh buffer has no position header",
        ))
    }

    fn preview_submesh(
        &self,
        submesh: &Submesh,
        integer_faces: bool,
        group_id: u8,
        submesh_index: usize,
        material_name: Option<&String>,
    ) -> ParseResult<PreviewSubmesh> {
        let vert_start = submesh.verts_index_offset.max(0) as usize;
        let vert_end = vert_start + submesh.vert_count.max(0) as usize;
        let positions = self
            .positions
            .get(vert_start..vert_end)
            .ok_or(ParseError::Unsupported(
                "submesh vertex range is outside the position buffer",
            ))?;
        let normals = if self.normals.is_empty() {
            Vec::new()
        } else {
            self.normals
                .get(vert_start..vert_end)
                .ok_or(ParseError::Unsupported(
                    "submesh vertex range is outside the normal buffer",
                ))?
                .to_vec()
        };
        let uvs = if self.uv0.is_empty() {
            Vec::new()
        } else {
            self.uv0
                .get(vert_start..vert_end)
                .ok_or(ParseError::Unsupported(
                    "submesh vertex range is outside the UV0 buffer",
                ))?
                .to_vec()
        };

        let face_start = submesh.faces_index_offset.max(0) as usize;
        let face_end = face_start + submesh.indices_count.max(0) as usize;
        let indices = if integer_faces {
            self.integer_faces
                .as_ref()
                .and_then(|faces| faces.get(face_start..face_end))
                .ok_or(ParseError::Unsupported(
                    "submesh face range is outside the u32 index buffer",
                ))?
                .to_vec()
        } else {
            self.faces
                .as_ref()
                .and_then(|faces| faces.get(face_start..face_end))
                .ok_or(ParseError::Unsupported(
                    "submesh face range is outside the u16 index buffer",
                ))?
                .iter()
                .map(|index| u32::from(*index))
                .collect()
        };

        Ok(PreviewSubmesh {
            name: format!(
                "Group_{group_id}_sub{submesh_index}__{}",
                material_name.map(String::as_str).unwrap_or("NO_MATERIAL")
            ),
            material_index: usize::from(submesh.material_index),
            positions: positions.to_vec(),
            normals,
            uvs,
            indices,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MeshBufferItemHeader {
    pub kind: VertexBufferType,
    pub size: u16,
    pub offset: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VertexBufferType {
    Position,
    NormalsTangents,
    Uv0,
    Uv1,
    BoneWeights,
    Colors,
    Uv2,
    BoneWeights2,
    Unknown(i16),
}

impl VertexBufferType {
    fn from_raw(value: i16) -> Self {
        match value {
            0 => Self::Position,
            1 => Self::NormalsTangents,
            2 => Self::Uv0,
            3 => Self::Uv1,
            4 => Self::BoneWeights,
            5 => Self::Colors,
            6 => Self::Uv2,
            7 => Self::BoneWeights2,
            other => Self::Unknown(other),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MeshData {
    pub lods: Vec<MeshLod>,
    pub lod_count: usize,
    pub material_count: usize,
    pub uv_count: u8,
    pub skin_weight_count: u8,
    pub total_mesh_count: i16,
    pub integer_faces: bool,
    pub bounding_sphere: [f32; 4],
    pub bounding_box_min: [f32; 4],
    pub bounding_box_max: [f32; 4],
}

impl MeshData {
    fn read(
        reader: &mut Reader<'_>,
        buffer: &MeshBuffer,
        serializer_version: MeshSerializerVersion,
    ) -> ParseResult<Self> {
        let lod_count = usize::from(reader.read_u8()?);
        let material_count = usize::from(reader.read_u8()?);
        let uv_count = reader.read_u8()?;
        let skin_weight_count = reader.read_u8()?;
        let total_mesh_count = reader.read_i16()?;
        let integer_faces = reader.read_bool()?;
        reader.skip(1)?;
        if serializer_version <= MeshSerializerVersion::Dmc5 {
            reader.skip(8)?;
        }
        let bounding_sphere = reader.read_vec4()?;
        let bounding_box_min = reader.read_vec4()?;
        let bounding_box_max = reader.read_vec4()?;
        let lod_offsets_start = reader.read_u64()?;
        let mut offset_reader = reader.fork_at(lod_offsets_start)?;
        let mut lod_offsets = Vec::with_capacity(lod_count);
        for _ in 0..lod_count {
            lod_offsets.push(offset_reader.read_u64()?);
        }

        let mut lods = Vec::with_capacity(lod_count);
        for offset in lod_offsets {
            let mut lod_reader = reader.fork_at(offset)?;
            lods.push(MeshLod::read(&mut lod_reader, buffer, serializer_version)?);
        }

        Ok(Self {
            lods,
            lod_count,
            material_count,
            uv_count,
            skin_weight_count,
            total_mesh_count,
            integer_faces,
            bounding_sphere,
            bounding_box_min,
            bounding_box_max,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MeshLod {
    pub mesh_groups: Vec<MeshGroup>,
    pub lod_factor: f32,
    pub vertex_format: u8,
}

impl MeshLod {
    fn read(
        reader: &mut Reader<'_>,
        buffer: &MeshBuffer,
        serializer_version: MeshSerializerVersion,
    ) -> ParseResult<Self> {
        let mesh_count = usize::from(reader.read_u8()?);
        let vertex_format = reader.read_u8()?;
        let _ukn1 = reader.read_u8()?;
        reader.skip(1)?;
        let lod_factor = reader.read_f32()?;
        let header_offset = reader.read_u64()?;
        let mut offset_reader = reader.fork_at(header_offset)?;
        let mut mesh_offsets = Vec::with_capacity(mesh_count);
        for _ in 0..mesh_count {
            mesh_offsets.push(offset_reader.read_u64()?);
        }

        let mut mesh_groups = Vec::with_capacity(mesh_count);
        for offset in mesh_offsets {
            let mut mesh_reader = reader.fork_at(offset)?;
            mesh_groups.push(MeshGroup::read(
                &mut mesh_reader,
                buffer,
                serializer_version,
            )?);
        }

        Ok(Self {
            mesh_groups,
            lod_factor,
            vertex_format,
        })
    }
}

#[derive(Debug, Clone)]
pub struct MeshGroup {
    pub group_id: u8,
    pub vertex_count: i32,
    pub indices_count: i32,
    pub submeshes: Vec<Submesh>,
}

impl MeshGroup {
    fn read(
        reader: &mut Reader<'_>,
        _buffer: &MeshBuffer,
        serializer_version: MeshSerializerVersion,
    ) -> ParseResult<Self> {
        let group_id = reader.read_u8()?;
        let submesh_count = usize::from(reader.read_u8()?);
        reader.skip(6)?;
        let vertex_count = reader.read_i32()?;
        let indices_count = reader.read_i32()?;

        let mut submeshes = Vec::with_capacity(submesh_count);
        for _ in 0..submesh_count {
            submeshes.push(Submesh::read(reader, serializer_version)?);
        }

        for index in 0..submeshes.len() {
            let next_offset = submeshes
                .get(index + 1)
                .map(|next| next.verts_index_offset)
                .unwrap_or_else(|| submeshes[0].verts_index_offset + vertex_count);
            submeshes[index].vert_count = next_offset - submeshes[index].verts_index_offset;
        }

        Ok(Self {
            group_id,
            vertex_count,
            indices_count,
            submeshes,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Submesh {
    pub material_index: u16,
    pub buffer_index: u8,
    pub indices_count: i32,
    pub faces_index_offset: i32,
    pub verts_index_offset: i32,
    pub streaming_offset: i32,
    pub streaming_offset2: i32,
    pub vert_count: i32,
}

impl Submesh {
    fn read(
        reader: &mut Reader<'_>,
        serializer_version: MeshSerializerVersion,
    ) -> ParseResult<Self> {
        let material_index = reader.read_u16()?;
        let buffer_index = reader.read_u8()?;
        let _ukn_byte = reader.read_u8()?;
        if serializer_version >= MeshSerializerVersion::Onimusha {
            let _ukn1 = reader.read_i32()?;
        }
        let indices_count = reader.read_i32()?;
        let faces_index_offset = reader.read_i32()?;
        let verts_index_offset = reader.read_i32()?;
        let (streaming_offset, streaming_offset2) =
            if serializer_version >= MeshSerializerVersion::ReRt {
                (reader.read_i32()?, reader.read_i32()?)
            } else {
                (0, 0)
            };
        if serializer_version >= MeshSerializerVersion::Dd2 {
            let _ukn2 = reader.read_i32()?;
        }
        Ok(Self {
            material_index,
            buffer_index,
            indices_count,
            faces_index_offset,
            verts_index_offset,
            streaming_offset,
            streaming_offset2,
            vert_count: 0,
        })
    }
}

fn read_strings(reader: &Reader<'_>, offset: u64, count: u16) -> ParseResult<Vec<String>> {
    if offset == 0 || count == 0 {
        return Ok(Vec::new());
    }
    let mut string_reader = reader.fork_at(offset)?;
    (0..count)
        .map(|_| string_reader.read_offset_ascii_string())
        .collect()
}

fn read_buffer_headers(
    reader: &mut Reader<'_>,
    count: usize,
) -> ParseResult<Vec<MeshBufferItemHeader>> {
    (0..count)
        .map(|_| {
            Ok(MeshBufferItemHeader {
                kind: VertexBufferType::from_raw(reader.read_i16()?),
                size: reader.read_u16()?,
                offset: reader.read_u32()?,
            })
        })
        .collect()
}

fn read_half2(reader: &mut Reader<'_>) -> ParseResult<[f32; 2]> {
    Ok([
        f16_to_f32(reader.read_u16()?),
        f16_to_f32(reader.read_u16()?),
    ])
}

fn read_quantized_normal(reader: &mut Reader<'_>) -> ParseResult<[f32; 3]> {
    let x = reader.read_i8()?;
    let y = reader.read_i8()?;
    let z = reader.read_i8()?;
    let _w = reader.read_i8()?;
    reader.skip(4)?;
    Ok([
        (((x as i32) << 1) + 1) as f32 / 255.0,
        (((y as i32) << 1) + 1) as f32 / 255.0,
        (((z as i32) << 1) + 1) as f32 / 255.0,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MESH: &str = "test_files/mhs3/natives/STM/Art/Model/character/ch00/000/0000/ch00_000_0000.mesh.250604100";

    #[test]
    #[ignore = "requires local RE Engine mesh fixture files"]
    fn reads_mhs3_mesh_header_lod_and_preview_geometry() {
        let mesh = MeshFile::read_path(SAMPLE_MESH).expect("mesh should parse");

        assert_eq!(mesh.header.version, 250203152);
        assert_eq!(
            mesh.header.serializer_version,
            MeshSerializerVersion::Pragmata
        );
        assert_eq!(mesh.header.buffer_count(), 1);
        assert!(!mesh.material_names.is_empty());

        let data = mesh.mesh_data.as_ref().expect("mesh data");
        assert!(!data.lods.is_empty());
        assert!(!data.lods[0].mesh_groups.is_empty());

        let buffer = mesh.mesh_buffer.as_ref().expect("mesh buffer");
        assert!(!buffer.positions.is_empty());
        assert!(buffer.faces.as_ref().is_some_and(|faces| !faces.is_empty()));

        let preview = mesh.to_preview_model().expect("preview model");
        assert!(!preview.meshes.is_empty());
        assert_eq!(preview.materials.len(), mesh.material_names.len());
        assert!(preview.meshes.iter().any(|sub| !sub.positions.is_empty()));
        assert!(preview.meshes.iter().any(|sub| !sub.indices.is_empty()));
        assert!(preview.meshes.iter().all(|sub| {
            sub.indices
                .iter()
                .all(|index| (*index as usize) < sub.positions.len())
        }));
    }
}
