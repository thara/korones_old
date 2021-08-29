use std::path::Path;

use anyhow::Result;

use crate::apu::*;
use crate::controller::*;
use crate::nes::Nes;
use crate::rom::Rom;

pub struct Emulator {
    nes: Nes,
}

impl Emulator {
    pub fn new(sampling_rate: u32, frame_period: u32) -> Self {
        Self {
            nes: Nes::new(sampling_rate, frame_period),
        }
    }

    pub fn set_audio_buffer(&mut self, audio_buffer: Box<dyn AudioBuffer>) {
        self.nes.apu.audio_buffer = audio_buffer;
    }

    pub fn step_frame(&mut self) {
        self.nes.step_frame();
    }

    pub fn set_controllers(&mut self, c1: Box<dyn Controller>, c2: Box<dyn Controller>) {
        self.nes.controller_1 = c1;
        self.nes.controller_2 = c2;
    }

    pub fn load_rom<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let rom = Rom::load_file(path)?;
        self.nes.set_rom(rom);
        self.nes.power_on();
        self.nes.clear();
        Ok(())
    }
}
