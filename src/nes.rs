use crate::cpu::*;
use crate::data_unit::*;
use crate::interrupt::*;

pub struct Nes {
    pub(crate) cpu: Cpu,
    pub(crate) interrupt: Interrupt,

    wram: [u8; 0x2000],
}

impl Default for Nes {
    fn default() -> Self {
        Nes {
            cpu: Cpu::default(),
            wram: [0; 0x2000],
        }
    }
}

impl Nes {
    pub(crate) fn read_bus(&mut self, addr: impl Into<Word>) -> Byte {
        let w = addr.into();
        let a: u16 = w.into();
        match a {
            0x0000..=0x1FFF => self.wram[a as usize].into(),
            // 0x2000..=0x3FFF => ppu::read_register(to_ppu_addr(a), nes),
            // 0x4000..=0x4013 | 0x4015 => nes.apu.read_status(),
            // 0x4016 => nes.controller_1.read(),
            // 0x4017 => nes.controller_2.read(),
            // 0x4020..=0xFFFF => nes.mapper.read(addr),
            _ => 0u8.into(),
        }
    }

    pub(crate) fn write_bus(&mut self, addr: impl Into<Word>, value: impl Into<Byte>) {
        let w = addr.into();
        let a: u16 = w.into();
        let v = value.into();
        match a {
            0x0000..=0x1FFF => self.wram[a as usize] = v.into(),
            // 0x2000..=0x3FFF => ppu::write_register(addr, value, nes),
            // 0x4000..=0x4013 | 0x4015 => nes.apu.write(addr, value),
            // 0x4016 => nes.controller_1.write(value),
            // 0x4017 => {
            //     nes.controller_2.write(value);
            //     nes.apu.write(addr, value);
            // }
            // 0x4020..=0xFFFF => nes.mapper.write(addr, value),
            _ => {
                //NOP
            }
        }
    }
}
