use crate::cpu;
use crate::nes;
use crate::ppu;

pub struct Emulator {
    nes: nes::Nes,

    scan: ppu::Scan,
    frames: u64,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            nes: Default::default(),
            scan: Default::default(),
            frames: Default::default(),
        }
    }

    pub fn step_frame(&mut self) {
        let before = self.frames;

        while before == self.frames {
            self.step();
        }
    }

    pub fn step(&mut self) {
        let cycles = cpu::step(&mut self.nes);

        let mut ppu_cycles = cycles * 3;
        while 0 < ppu_cycles {
            self.frames = ppu::step(&mut self.nes, &mut self.scan, self.frames);
            ppu_cycles -= 1;
        }
    }
}
