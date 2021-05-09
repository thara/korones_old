use crate::prelude::*;

pub enum Mirroring {
    Vertical,
    Horizontal,
}

pub trait Mapper {
    fn read(&self, addr: Word) -> Byte;
    fn write(&self, addr: Word, value: Byte);
    fn mirroring(&self) -> Mirroring;
}

pub struct MapperDefault {}

impl Mapper for MapperDefault {
    fn read(&self, _: Word) -> Byte {
        Default::default()
    }

    fn write(&self, _: Word, _: Byte) {
        // NOP
    }

    fn mirroring(&self) -> Mirroring {
        Mirroring::Vertical
    }
}
