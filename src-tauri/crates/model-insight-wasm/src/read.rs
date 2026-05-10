use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};

pub type ParseResult<T> = Result<T, ParseError>;

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("invalid magic: expected {expected}, got 0x{actual:08x}")]
    InvalidMagic { expected: &'static str, actual: u32 },
    #[error("unsupported format: {0}")]
    Unsupported(&'static str),
    #[error(
        "read out of bounds at {offset:#x}: requested {requested} bytes from {len} byte buffer"
    )]
    OutOfBounds {
        offset: usize,
        len: usize,
        requested: usize,
    },
    #[error("invalid file offset {0:#x}")]
    InvalidOffset(u64),
    #[error("invalid utf-16 string")]
    InvalidUtf16,
    #[error("missing numeric RE file version suffix")]
    MissingFileVersion,
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

#[derive(Clone, Copy)]
pub struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn seek(&mut self, offset: u64) -> ParseResult<()> {
        let offset = usize::try_from(offset).map_err(|_| ParseError::InvalidOffset(offset))?;
        if offset > self.data.len() {
            return Err(ParseError::InvalidOffset(offset as u64));
        }
        self.pos = offset;
        Ok(())
    }

    pub fn skip(&mut self, bytes: usize) -> ParseResult<()> {
        self.read_bytes(bytes).map(|_| ())
    }

    pub fn fork_at(&self, offset: u64) -> ParseResult<Self> {
        let mut fork = *self;
        fork.seek(offset)?;
        Ok(fork)
    }

    pub fn read_u8(&mut self) -> ParseResult<u8> {
        Ok(self.read_bytes(1)?[0])
    }

    pub fn read_i8(&mut self) -> ParseResult<i8> {
        Ok(self.read_u8()? as i8)
    }

    pub fn read_bool(&mut self) -> ParseResult<bool> {
        Ok(self.read_u8()? != 0)
    }

    pub fn read_u16(&mut self) -> ParseResult<u16> {
        Ok(LittleEndian::read_u16(self.read_bytes(2)?))
    }

    pub fn read_i16(&mut self) -> ParseResult<i16> {
        Ok(LittleEndian::read_i16(self.read_bytes(2)?))
    }

    pub fn read_u32(&mut self) -> ParseResult<u32> {
        Ok(LittleEndian::read_u32(self.read_bytes(4)?))
    }

    pub fn read_i32(&mut self) -> ParseResult<i32> {
        Ok(LittleEndian::read_i32(self.read_bytes(4)?))
    }

    pub fn read_u64(&mut self) -> ParseResult<u64> {
        Ok(LittleEndian::read_u64(self.read_bytes(8)?))
    }

    pub fn read_f32(&mut self) -> ParseResult<f32> {
        Ok(f32::from_bits(self.read_u32()?))
    }

    pub fn read_vec3(&mut self) -> ParseResult<[f32; 3]> {
        Ok([self.read_f32()?, self.read_f32()?, self.read_f32()?])
    }

    pub fn read_vec4(&mut self) -> ParseResult<[f32; 4]> {
        Ok([
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
            self.read_f32()?,
        ])
    }

    pub fn read_ascii_cstr_at(&self, offset: u64) -> ParseResult<String> {
        let offset = usize::try_from(offset).map_err(|_| ParseError::InvalidOffset(offset))?;
        if offset >= self.data.len() {
            return Err(ParseError::InvalidOffset(offset as u64));
        }
        let end = self.data[offset..]
            .iter()
            .position(|&b| b == 0)
            .map(|relative| offset + relative)
            .unwrap_or(self.data.len());
        Ok(String::from_utf8_lossy(&self.data[offset..end]).into_owned())
    }

    pub fn read_utf16_cstr_at(&self, offset: u64) -> ParseResult<String> {
        let offset = usize::try_from(offset).map_err(|_| ParseError::InvalidOffset(offset))?;
        if offset >= self.data.len() || offset % 2 != 0 {
            return Err(ParseError::InvalidOffset(offset as u64));
        }
        let mut words = Vec::new();
        let mut cursor = offset;
        loop {
            if cursor + 2 > self.data.len() {
                return Err(ParseError::OutOfBounds {
                    offset: cursor,
                    len: self.data.len(),
                    requested: 2,
                });
            }
            let word = u16::from_le_bytes([self.data[cursor], self.data[cursor + 1]]);
            cursor += 2;
            if word == 0 {
                break;
            }
            words.push(word);
        }
        String::from_utf16(&words).map_err(|_| ParseError::InvalidUtf16)
    }

    pub fn read_offset_ascii_string(&mut self) -> ParseResult<String> {
        let offset = self.read_u64()?;
        self.read_ascii_cstr_at(offset)
    }

    pub fn read_offset_utf16_string(&mut self) -> ParseResult<String> {
        let offset = self.read_u64()?;
        if offset == 0 {
            return Ok(String::new());
        }
        self.read_utf16_cstr_at(offset)
    }

    pub fn read_bytes(&mut self, len: usize) -> ParseResult<&'a [u8]> {
        let end = self.pos.checked_add(len).ok_or(ParseError::OutOfBounds {
            offset: self.pos,
            len: self.data.len(),
            requested: len,
        })?;
        if end > self.data.len() {
            return Err(ParseError::OutOfBounds {
                offset: self.pos,
                len: self.data.len(),
                requested: len,
            });
        }
        let bytes = &self.data[self.pos..end];
        self.pos = end;
        Ok(bytes)
    }
}

pub fn read_file(path: impl AsRef<Path>) -> ParseResult<Vec<u8>> {
    Ok(std::fs::read(path)?)
}

pub fn file_version_from_path(path: impl AsRef<Path>, marker: &str) -> ParseResult<u32> {
    let filename = path
        .as_ref()
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or(ParseError::MissingFileVersion)?;
    let needle = format!("{marker}.");
    let (_, version) = filename
        .rsplit_once(&needle)
        .ok_or(ParseError::MissingFileVersion)?;
    version
        .parse::<u32>()
        .map_err(|_| ParseError::MissingFileVersion)
}

pub fn f16_to_f32(bits: u16) -> f32 {
    let sign = ((bits & 0x8000) as u32) << 16;
    let exponent = (bits & 0x7c00) >> 10;
    let mantissa = (bits & 0x03ff) as u32;

    let f_bits = match exponent {
        0 if mantissa == 0 => sign,
        0 => {
            let mut mant = mantissa;
            let mut exp = -14i32;
            while (mant & 0x0400) == 0 {
                mant <<= 1;
                exp -= 1;
            }
            mant &= 0x03ff;
            sign | (((exp + 127) as u32) << 23) | (mant << 13)
        }
        0x1f if mantissa == 0 => sign | 0x7f80_0000,
        0x1f => sign | 0x7f80_0000 | (mantissa << 13),
        _ => sign | (((exponent as u32) + 112) << 23) | (mantissa << 13),
    };

    f32::from_bits(f_bits)
}
