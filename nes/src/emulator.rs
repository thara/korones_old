use crate::controller::Controller;
use crate::cpu::{self, handle_interrupt, CpuClock, Trace};
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

    pub fn join_controller(
        &mut self,
        controller_1: Box<dyn Controller>,
        controller_2: Box<dyn Controller>,
    ) {
        self.nes.controller_1 = controller_1;
        self.nes.controller_2 = controller_2;
    }

    pub fn update_controller_1(&mut self, state: u8) {
        self.nes.controller_1.update(state.into());
    }

    pub fn update_controller_2(&mut self, state: u8) {
        self.nes.controller_2.update(state.into());
    }
}

// nestest
impl Emulator {
    pub fn nestest<F: FnMut(&Trace)>(&mut self, mut f: F) {
        // initial state
        self.nes.cpu.init_nestest();
        self.nes.cpu_cycles = 7;
        for _ in 0..7 {
            ppu::step(&mut self.nes);
            ppu::step(&mut self.nes);
            ppu::step(&mut self.nes);
        }

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
        ppu::step(nes);
        ppu::step(nes);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::controller::StandardController;
    use std::fs::File;
    use std::io::{self, BufRead};
    use std::path::Path;

    #[test]
    fn nestest() {
        let nes_dir = env!("CARGO_MANIFEST_DIR");

        let rom_path = Path::new(nes_dir)
            .parent()
            .unwrap()
            .join("roms/nes-test-roms/other/nestest.nes");

        let cart = Cartridge::load_rom_file(rom_path).unwrap();

        let mut emu = Emulator::new();

        let controller_1 = Box::new(StandardController::default());
        let controller_2 = Box::new(StandardController::default());
        emu.join_controller(controller_1, controller_2);

        emu.insert_cartridge(cart);
        emu.power_on();

        let log_path = Path::new(nes_dir)
            .parent()
            .unwrap()
            .join("roms/nestest-cpu.log");

        let f = File::open(log_path).unwrap();
        let mut lines = io::BufReader::new(f).lines();

        emu.nestest(|trace| {
            let line = lines.next().unwrap().unwrap();
            assert_eq!(format!("{}", trace), line);
        });
    }
}
