use crate::apu::Apu;
use crate::controller::{self, Controller};
use crate::cpu::Cpu;
use crate::interrupt::Interrupt;
use crate::mapper::{Mapper, MapperDefault};
use crate::ppu::{self, Ppu};
use crate::prelude::*;

pub struct Nes {
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub apu: Apu,

    pub interrupt: Interrupt,

    pub mapper: Box<dyn Mapper>,

    pub controller_1: Box<dyn Controller>,
    pub controller_2: Box<dyn Controller>,
}

impl Nes {
    pub fn new() -> Self {
        Self {
            cpu: Cpu::new(),
            interrupt: Default::default(),
            ppu: Ppu::new(),
            apu: Apu::new(1_789_772 / 44100, 7458),
            mapper: Box::new(MapperDefault {}),
            controller_1: Box::new(controller::Empty {}),
            controller_2: Box::new(controller::Empty {}),
        }
    }
}

pub struct SystemBus {}

impl Bus for SystemBus {
    fn read(addr: Word, nes: &mut Nes) -> Byte {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.cpu.wram[a as usize].into(),
            0x2000..=0x3FFF => ppu::read_register(to_ppu_addr(a), nes),
            0x4000..=0x4013 | 0x4015 => nes.apu.read_status(),
            0x4016 => nes.controller_1.read(),
            0x4017 => nes.controller_2.read(),
            0x4020..=0xFFFF => nes.mapper.read(addr),
            _ => 0u8.into(),
        }
    }

    fn write(addr: Word, value: Byte, nes: &mut Nes) {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.cpu.wram[a as usize] = value.into(),
            0x2000..=0x3FFF => ppu::write_register(addr, value, nes),
            0x4000..=0x4013 | 0x4015 => nes.apu.write(addr, value),
            0x4016 => nes.controller_1.write(value),
            0x4017 => {
                nes.controller_2.write(value);
                nes.apu.write(addr, value);
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
