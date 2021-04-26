bitflags! {
    #[derive(Default)]
    pub struct Interrupt: u8 {
        const RESET = 1 << 3;
        const NMI = 1 << 2;
        const IRQ = 1 << 1;
        const BRK = 1 << 0;

        const NO_INTERRUPT = 0;
    }
}

impl Interrupt {
    pub fn get(&self) -> Self {
        if self.contains(Self::RESET) {
            Self::RESET
        } else if self.contains(Self::NMI) {
            Self::NMI
        } else if self.contains(Self::IRQ) {
            Self::IRQ
        } else if self.contains(Self::BRK) {
            Self::BRK
        } else {
            Self::NO_INTERRUPT
        }
    }
}
