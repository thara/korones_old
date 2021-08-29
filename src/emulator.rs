use crate::nes;

pub struct Emulator {
    nes: nes::Nes,
}

impl Emulator {
    pub fn new() -> Self {
        Self {
            nes: Default::default(),
        }
    }

    pub fn step_frame(&mut self) {
        self.nes.step_frame();
    }
}
