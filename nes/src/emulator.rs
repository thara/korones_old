use crate::cpu;
use crate::nes::{Nes, SystemBus, SystemClock};

pub struct Emulator {
    nes: Nes,
}

impl Emulator {
    pub fn new() -> Self {
        Self { nes: Nes::new() }
    }

    pub fn step(&mut self) {
        cpu::step::<SystemBus, SystemClock>(&mut self.nes);
    }
}
