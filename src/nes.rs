use crate::cpu::*;
use crate::data_unit::*;
use crate::interrupt::*;
use crate::ppu::*;
use crate::rom::*;

const HEIGHT: usize = 240;
const WIDTH: usize = 256;

type FrameBuffer = [u8; 240 * 256];

pub struct Nes {
    pub(crate) cpu: Cpu,
    wram: [u8; 0x2000],
    pub(crate) interrupt: Interrupt,

    pub(crate) ppu: Ppu,
    pub(crate) oam: Oam,
    pub(crate) name_table: [Byte; 0x1000],
    pub(crate) pallete_ram_idx: [Byte; 0x0020],

    pub(crate) mapper: Box<dyn Mapper>,

    buffers: [FrameBuffer; 2],
    buffer_index: usize,
}

impl Default for Nes {
    fn default() -> Self {
        Nes {
            cpu: Cpu::default(),
            wram: [0; 0x2000],
            interrupt: Interrupt::default(),
            ppu: Ppu::default(),
            oam: Oam::new(),
            name_table: [Default::default(); 0x1000],
            pallete_ram_idx: [Default::default(); 0x0020],
            mapper: Box::new(MapperDefault {}),
            buffers: [[0; 240 * 256], [0; 240 * 256]],
            buffer_index: 0,
        }
    }
}

impl Nes {
    pub(crate) fn read_bus(&mut self, addr: impl Into<Word>) -> Byte {
        let w = addr.into();
        let a: u16 = w.into();
        match a {
            0x0000..=0x1FFF => self.wram[a as usize].into(),
            0x2000..=0x3FFF => self.read_ppu_register(to_ppu_addr(a)),
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
            0x2000..=0x3FFF => self.write_ppu_register(to_ppu_addr(a), v),
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

fn to_ppu_addr(addr: u16) -> u16 {
    // repears every 8 bytes
    0x2000u16.wrapping_add(addr) % 8
}

// frame buffers
impl Nes {
    pub fn current_buffer(&mut self) -> &FrameBuffer {
        &mut self.buffers[self.buffer_index]
    }

    pub(crate) fn write_buffer(&mut self, x: usize, y: usize, color: Byte) {
        let b = &mut self.buffers[(self.buffer_index + 1) % 2];
        b[y * WIDTH + x] = color.into();
    }

    pub(crate) fn swap_buffers(&mut self) {
        self.buffer_index = (self.buffer_index + 1) % 2;
    }
}
