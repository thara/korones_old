use crate::nes::Nes;

pub struct Emulator {
    nes: Nes,
}

impl Emulator {
    pub fn new(sampling_rate: u32, frame_period: u32) -> Self {
        Self {
            nes: Nes::new(sampling_rate, frame_period),
        }
    }

    pub fn step_frame(&mut self) {
        self.nes.step_frame();
    }
}
