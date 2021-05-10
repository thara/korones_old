mod inesfile;
mod mapper_0;

use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result};
use thiserror::Error;

use crate::prelude::*;

#[derive(Debug, Clone)]
pub enum Mirroring {
    Vertical,
    Horizontal,
}

pub trait Mapper {
    fn read(&mut self, addr: Word) -> Byte;
    fn write(&mut self, addr: Word, value: Byte);
    fn mirroring(&self) -> Mirroring;
}

pub struct MapperDefault {}

impl Mapper for MapperDefault {
    fn read(&mut self, _: Word) -> Byte {
        Default::default()
    }

    fn write(&mut self, _: Word, _: Byte) {
        // NOP
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}

pub struct Cartridge {
    mapper_no: u8,
    pub(crate) mapper: Box<dyn Mapper>,
}

impl Cartridge {
    pub fn from_rom_data(data: Vec<u8>) -> Result<Self> {
        use binread::{io::Cursor, BinRead};

        use self::inesfile::INESFile;
        use self::mapper_0::Mapper0;

        let mut cur = Cursor::new(data);
        let ines = INESFile::read(&mut cur)?;

        let mapper_no = ines.mapper;
        let mapper = match ines.mapper {
            0 => Ok(Mapper0::new(ines)),
            no @ _ => Err(CartridgeError::NotSupportedMapper(no)),
        }?;

        Ok(Self {
            mapper_no,
            mapper: Box::new(mapper),
        })
    }

    pub fn load_rom_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        use std::io::Read;

        let mut f = File::open(path.as_ref()).with_context(|| {
            format!(
                "Failed to open INES file: {}",
                path.as_ref().to_str().unwrap_or("unknown")
            )
        })?;

        let mut buf = Vec::new();
        let _ = f.read_to_end(&mut buf);
        Self::from_rom_data(buf)
    }
}

impl std::fmt::Debug for Cartridge {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(fmt, "Cartridge {{ mapper: {:?} }}", self.mapper_no)
    }
}

#[derive(Debug, Error)]
pub enum CartridgeError {
    #[error("mapper `{0}` are not supported")]
    NotSupportedMapper(u8),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_rom_file() {
        let nes_dir = env!("CARGO_MANIFEST_DIR");
        let path = Path::new(nes_dir)
            .parent()
            .unwrap()
            .join("roms/nes-test-roms/other/nestest.nes");

        let result = Cartridge::load_rom_file(path);
        assert_matches!(result, Ok(_));
    }
}
