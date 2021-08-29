use crate::nes::*;

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

pub fn handle_interrupt(nes: &mut Nes) -> u128 {
    let before = nes.cpu.cycles;

    let current = nes.interrupt.get();
    match current {
        Interrupt::RESET => {
            nes.reset();
            nes.interrupt.remove(current)
        }
        Interrupt::NMI => {
            nes.non_markable_interrupt();
            nes.interrupt.remove(current)
        }
        Interrupt::IRQ => {
            if nes.cpu.interrupted() {
                nes.interrupt_request();
                nes.interrupt.remove(current)
            }
        }
        Interrupt::BRK => {
            if nes.cpu.interrupted() {
                nes.break_interrupt();
                nes.interrupt.remove(current)
            }
        }
        _ => {}
    }

    if before <= nes.cpu.cycles {
        nes.cpu.cycles - before
    } else {
        u128::MAX - before + nes.cpu.cycles
    }
}
