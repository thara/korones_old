use crate::cpu::{self, handle_interrupt, Cpu, Trace};
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

// nestest
impl Emulator {
    pub fn nestest<F: FnMut(&Trace)>(&mut self, mut f: F) {
        // initial state
        self.nes.cpu_cycles = 7;
        self.nes.cpu = Cpu::nestest();

        loop {
            handle_interrupt::<SystemBus, SystemClock>(&mut self.nes);

            let trace = Trace::new::<SystemBus>(&mut self.nes);
            f(&trace);

            cpu::step::<SystemBus, SystemClock>(&mut self.nes);

            if 26554 < self.nes.cpu_cycles {
                break;
            }
        }
    }
}
