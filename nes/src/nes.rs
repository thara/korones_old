use crate::cpu::Cpu;
use crate::interrupt::Interrupt;
use crate::mapper::{Mapper, MapperDefault};
use crate::ppu::{self, Ppu};
use crate::prelude::*;

pub struct Nes {
    // CPU
    pub cpu: Cpu,
    pub wram: [u8; 0x2000],
    pub cpu_cycles: u128,

    pub interrupt: Interrupt,

    // PPU
    pub ppu: Ppu,

    pub cycles: u128,

    pub mapper: Box<dyn Mapper>,
}

impl Nes {
    pub fn new() -> Self {
        Self {
            cpu: Default::default(),
            wram: [0; 0x2000],
            cpu_cycles: 0,
            interrupt: Default::default(),
            ppu: Ppu::new(),
            cycles: 0,
            mapper: Box::new(MapperDefault {}),
        }
    }
}

pub struct SystemBus {}

impl Bus for SystemBus {
    fn read(addr: Word, nes: &mut Nes) -> Byte {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.wram[a as usize].into(),
            0x2000..=0x3FFF => ppu::read_register(to_ppu_addr(a), nes),
            0x4000..=0x4013 | 0x4015 => {
                //TODO APU
                0u8.into()
            }
            0x4016 | 0x4017 => {
                //TODO controllers
                0u8.into()
            }
            0x4020..=0xFFFF => nes.mapper.read(addr),
            _ => 0u8.into(),
        }
    }

    fn write(addr: Word, value: Byte, nes: &mut Nes) {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.wram[a as usize] = value.into(),
            0x2000..=0x3FFF => ppu::write_register(addr, value, nes),
            0x4000..=0x4013 | 0x4015 => {
                //TODO APU
            }
            0x4016 => {
                //TODO controller 1
            }
            0x4017 => {
                //TODO controller 2 & APU
            }
            0x4020..=0xFFFF => nes.mapper.write(addr, value),
            _ => {
                //NOP
            }
        }
    }
}

fn to_ppu_addr(addr: u16) -> u16 {
    // repears every 8 bytes
    0x2000u16.wrapping_add(addr) % 8
}
