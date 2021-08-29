use crate::apu::{self, *};
use crate::controller::{self, Controller};
use crate::cpu::{self, *};
use crate::data_unit::*;
use crate::interrupt::*;
use crate::ppu::{self, *};
use crate::rom::*;

const HEIGHT: usize = 240;
const WIDTH: usize = 256;

const FRAME_BUFFER_LEN: usize = (ppu::MAX_LINE as usize) * (ppu::MAX_DOT as usize);
type FrameBuffer = [u8; FRAME_BUFFER_LEN];

pub struct Nes {
    pub(crate) cpu: Cpu,
    wram: [u8; 0x2000],
    pub(crate) interrupt: Interrupt,

    pub(crate) ppu: Ppu,
    pub(crate) oam: Oam,
    pub(crate) name_table: [Byte; 0x1000],
    pub(crate) pallete_ram_idx: [Byte; 0x0020],

    pub(crate) apu: Apu,

    pub(crate) mapper: Box<dyn Mapper>,
    pub(crate) controller_1: Box<dyn Controller>,
    pub(crate) controller_2: Box<dyn Controller>,

    buffers: [FrameBuffer; 2],
    buffer_index: usize,
}

impl Nes {
    pub(crate) fn new(sampling_rate: u32, frame_period: u32) -> Self {
        Self {
            apu: Apu::new(sampling_rate, frame_period),
            ..Default::default()
        }
    }

    pub(crate) fn clear(&mut self) {
        self.interrupt.remove(Interrupt::NMI | Interrupt::IRQ);
        self.interrupt.insert(Interrupt::RESET);
        self.wram = [0; 0x2000];
        self.ppu = Ppu::default();
    }
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
            apu: Apu::new(0, 0),
            controller_1: Box::new(controller::Empty {}),
            controller_2: Box::new(controller::Empty {}),
            buffers: [[0; FRAME_BUFFER_LEN], [0; FRAME_BUFFER_LEN]],
            buffer_index: 0,
        }
    }
}

impl Nes {
    pub fn step_frame(&mut self) {
        let before = self.ppu.frames;

        while before == self.ppu.frames {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let mut cycles = if self.cpu.interrupted() {
            handle_interrupt(self)
        } else {
            cpu::step(self)
        };

        for _ in 0..cycles {
            let cpu_steel = apu::step(self);
            cycles += cpu_steel;
        }

        let mut ppu_cycles = cycles * 3;
        while 0 < ppu_cycles {
            ppu::step(self);
            ppu_cycles -= 1;
        }
    }

    pub(crate) fn set_rom(&mut self, rom: Rom) {
        self.mapper = rom.mapper;
        self.ppu.mirroring = self.mapper.mirroring();
    }
}

impl Nes {
    pub(crate) fn read_bus(&mut self, addr: impl Into<Word>) -> Byte {
        let addr = addr.into();
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => self.wram[a as usize].into(),
            0x2000..=0x3FFF => self.read_ppu_register(to_ppu_addr(a)),
            0x4000..=0x4013 | 0x4015 => self.apu.read_status(),
            0x4016 => self.controller_1.read(),
            0x4017 => self.controller_2.read(),
            0x4020..=0xFFFF => self.mapper.read(addr),
            _ => 0u8.into(),
        }
    }

    pub(crate) fn write_bus(&mut self, addr: impl Into<Word>, value: impl Into<Byte>) {
        let addr = addr.into();
        let a: u16 = addr.into();
        let v = value.into();
        match a {
            0x0000..=0x1FFF => self.wram[a as usize] = v.into(),
            0x2000..=0x3FFF => self.write_ppu_register(to_ppu_addr(a), v),
            0x4000..=0x4013 | 0x4015 => self.apu.write(addr, v),
            0x4016 => self.controller_1.write(v),
            0x4017 => {
                self.controller_2.write(v);
                self.apu.write(addr, v);
            }
            0x4020..=0xFFFF => self.mapper.write(addr, v),
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::cpu;
    use crate::trace::*;

    use crate::controller::StandardController;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    impl Nes {
        fn nestest<F: FnMut(&Trace)>(&mut self, mut f: F) {
            // initial state
            self.cpu.pc = 0xC000u16.into();
            // https://wiki.nesdev.com/w/index.php/CPU_power_up_state#cite_ref-1
            self.cpu.p = cpu::Status::from_bits_truncate(0x24);
            self.cpu.cycles = 7;
            for _ in 0..7 {
                ppu::step(self);
                ppu::step(self);
                ppu::step(self);
            }

            loop {
                handle_interrupt(self);

                let trace = Trace::new(self);
                f(&trace);

                let cycles = cpu::step(self);

                let mut ppu_cycles = cycles * 3;
                while 0 < ppu_cycles {
                    ppu::step(self);
                    ppu_cycles -= 1;
                }

                if 26554 < self.cpu.cycles {
                    break;
                }
            }
        }
    }

    #[test]
    fn nestest() {
        let nes_dir = env!("CARGO_MANIFEST_DIR");

        let rom_path = Path::new(nes_dir).join("roms/nes-test-roms/other/nestest.nes");
        let rom = Rom::load_file(rom_path).unwrap();

        let mut nes: Nes = Default::default();

        nes.controller_1 = Box::new(StandardController::default());
        nes.controller_2 = Box::new(StandardController::default());

        nes.set_rom(rom);
        nes.power_on();

        let log_path = Path::new(nes_dir).join("roms/nestest-cpu.log");

        let f = File::open(log_path).unwrap();
        let mut lines = io::BufReader::new(f).lines();

        nes.nestest(|trace| {
            let line = lines.next().unwrap().unwrap();
            assert_eq!(format!("{}", trace), line);
        });
    }
}
