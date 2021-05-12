use crate::cpu::{self, handle_interrupt, Cpu, CpuClock, Trace};
use crate::mapper::Cartridge;
use crate::nes::{Nes, SystemBus};
use crate::ppu;

pub struct Emulator {
    nes: Nes,
}

impl Emulator {
    pub fn new() -> Self {
        Self { nes: Nes::new() }
    }

    pub fn power_on(&mut self) {
        cpu::power_on::<SystemBus>(&mut self.nes);
        self.nes.ppu.power_on();
    }

    pub fn step(&mut self) {
        cpu::step::<SystemBus, SystemClock>(&mut self.nes);
    }

    pub fn insert_cartridge(&mut self, cart: Cartridge) {
        self.nes.mapper = cart.mapper;

        cpu::reset::<SystemBus>(&mut self.nes);

        self.nes.ppu.reset();
        self.nes.ppu.mirroring = self.nes.mapper.mirroring();
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

pub struct SystemClock {}

impl CpuClock for SystemClock {
    fn tick(nes: &mut Nes) {
        nes.cpu_cycles = nes.cpu_cycles.wrapping_add(1);

        ppu::step(nes);
    }
}
