use crate::cpu;
use crate::nes;

pub struct Emulator {
    nes: nes::Nes,
}

impl Emulator {
    pub fn step(&mut self) {
        cpu::step(&mut self.nes);
    }
}
