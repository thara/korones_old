use crate::cpu::*;
use crate::nes;

pub struct Emulator {
    nes: nes::Nes,
}

impl Emulator {
    pub fn step(&mut self) {
        self.nes.cpu = self.nes.step(self.nes.cpu.clone());
    }
}
