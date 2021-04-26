use crate::cpu::{Cpu, CpuClock};
use crate::interrupt::Interrupt;
use crate::prelude::*;

pub struct Nes {
    // CPU
    pub cpu: Cpu,
    pub wram: [u8; 0x2000],
    pub cpu_cycles: u128,

    pub interrupt: Interrupt,

    pub cycles: u128,
}

impl Nes {
    pub fn new() -> Self {
        Self {
            cpu: Default::default(),
            wram: [0; 0x2000],
            cpu_cycles: 0,
            interrupt: Default::default(),
            cycles: 0,
        }
    }
}

pub struct SystemBus {}

impl Bus for SystemBus {
    fn read(addr: Word, nes: &mut Nes) -> Byte {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.wram[a as usize].into(),
            _ => unimplemented!(),
        }
    }

    fn write(addr: Word, value: Byte, nes: &mut Nes) {
        let a: u16 = addr.into();
        match a {
            0x0000..=0x1FFF => nes.wram[a as usize] = value.into(),
            _ => unimplemented!(),
        }
    }
}

pub struct SystemClock {}

impl CpuClock for SystemClock {
    fn tick(nes: &mut Nes) {
        nes.cpu_cycles = nes.cpu_cycles.wrapping_add(1);
    }
}
