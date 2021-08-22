use super::inesfile::{Flag6, INESFile};
use super::*;
use crate::prelude::*;

pub struct Mapper0 {
    rom: INESFile,

    mirroring: Mirroring,
    mirrored: bool,
}

impl Mapper0 {
    pub(super) fn new(rom: INESFile) -> Self {
        let mirroring = if rom.flag6.contains(Flag6::MIRRORING_VERTICAL) {
            Mirroring::Vertical
        } else {
            Mirroring::Horizontal
        };

        let mirrored = rom.prg_rom.len() == 0x4000;

        Self {
            rom: rom,
            mirroring,
            mirrored,
        }
    }

    fn prg_addr(&self, addr: u16) -> usize {
        if self.mirrored {
            addr.wrapping_rem(0x4000)
        } else {
            addr.wrapping_sub(0x4000)
        }
        .into()
    }
}

impl Mapper for Mapper0 {
    fn read(&mut self, addr: Word) -> Byte {
        let addr: u16 = addr.into();
        match addr {
            0x0000..=0x1FFF => self.rom.chr_rom[addr as usize],
            0x8000..=0xFFFF => self.rom.prg_rom[self.prg_addr(addr)],
            _ => 0,
        }
        .into()
    }

    fn write(&mut self, addr: Word, value: Byte) {
        let addr: u16 = addr.into();
        match addr {
            0x0000..=0x1FFF => self.rom.chr_rom[addr as usize] = value.into(),
            _ => {}
        }
    }

    fn mirroring(&self) -> Mirroring {
        self.mirroring.clone()
    }
}
