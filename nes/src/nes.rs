use crate::cpu::Cpu;
use crate::interrupt::Interrupt;
use crate::ppu::{self, Ppu, Scan};
use crate::prelude::*;

pub struct Nes {
    // CPU
    pub cpu: Cpu,
    pub wram: [u8; 0x2000],
    pub cpu_cycles: u128,

    pub interrupt: Interrupt,

    // PPU
    pub ppu: Ppu,
    pub scan: Scan,
    pub frames: u64,

    pub cycles: u128,

    pub mirroring: Mirroring,
}

impl Nes {
    pub fn new() -> Self {
        Self {
            cpu: Default::default(),
            wram: [0; 0x2000],
            cpu_cycles: 0,
            interrupt: Default::default(),
            ppu: Ppu::new(),
            scan: Default::default(),
            frames: 0,
            cycles: 0,
            mirroring: Mirroring::Vertical,
        }
    }
}

pub enum Mirroring {
    Vertical,
    Horizontal,
}

pub struct SystemBus {}

impl Bus for SystemBus {
    fn read(addr: Word, nes: &mut Nes) -> Byte {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.wram[a as usize].into(),
            0x2000..=0x3FFF => ppu::read_register(to_ppu_addr(a), nes),
            _ => unimplemented!(),
        }
    }

    fn write(addr: Word, value: Byte, nes: &mut Nes) {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.wram[a as usize] = value.into(),
            0x2000..=0x3FFF => ppu::write_register(addr, value, nes),
            _ => unimplemented!(),
        }
    }
}

fn to_ppu_addr(addr: u16) -> u16 {
    // repears every 8 bytes
    0x2000u16.wrapping_add(addr) % 8
}
