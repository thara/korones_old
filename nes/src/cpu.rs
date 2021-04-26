mod instruction;
mod interrupt;

use crate::interrupt::Interrupt;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Cpu {
    a: Byte,
    x: Byte,
    y: Byte,
    s: Byte,
    p: Status,
    pc: Word,
}

impl Cpu {
    pub fn interrupted(&self) -> bool {
        self.p.contains(Status::I)
    }
}

bitflags! {
    #[derive(Default)]
    struct Status: u8 {
        // Negative
        const N = 1 << 7;
        // Overflow
        const V = 1 << 6;
        const R = 1 << 5;
        const B = 1 << 4;
        // Decimal mode
        const D = 1 << 3;
        // IRQ prevention
        const I = 1 << 2;
        // Zero
        const Z = 1 << 1;
        // Carry
        const C = 1 << 0;
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        const OPERATED_B = 0b110000;
        const INTERRUPTED_B = 0b100000;
    }
}

pub trait CpuClock {
    fn tick(nes: &mut Nes);
}

pub fn step<M: Bus, C: CpuClock>(nes: &mut Nes) {
    use self::instruction::{decode, execute};

    let opcode = fetch::<CpuBus<M, C>>(nes);
    let instruction = decode(opcode);
    execute::<CpuBus<M, C>, C>(nes, instruction);
}

pub fn handle_interrupt<M: Bus, C: CpuClock>(nes: &mut Nes) {
    let current = nes.interrupt.get();
    match current {
        Interrupt::RESET => {
            interrupt::reset::<CpuBus<M, C>>(nes);
            nes.interrupt.remove(current)
        }
        Interrupt::NMI => {
            interrupt::non_markable_interrupt::<CpuBus<M, C>>(nes);
            nes.interrupt.remove(current)
        }
        Interrupt::IRQ => {
            if nes.cpu.interrupted() {
                interrupt::interrupt_request::<CpuBus<M, C>>(nes);
                nes.interrupt.remove(current)
            }
        }
        Interrupt::BRK => {
            if nes.cpu.interrupted() {
                interrupt::break_interrupt::<CpuBus<M, C>>(nes);
                nes.interrupt.remove(current)
            }
        }
        _ => {}
    }
}

fn fetch<M: Bus>(nes: &mut Nes) -> Byte {
    let opcode = M::read(nes.cpu.pc, nes);
    nes.cpu.pc += 1;
    opcode
}

// http://wiki.nesdev.com/w/index.php/CPU_addressing_modes
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[rustfmt::skip]
enum AddressingMode {
    Implicit,
    Accumulator,
    Immediate,
    ZeroPage, ZeroPageX, ZeroPageY,
    Absolute,
    AbsoluteX { penalty: bool },
    AbsoluteY { penalty: bool },
    Relative,
    Indirect, IndexedIndirect, IndirectIndexed,
}

// http://obelisk.me.uk/6502/reference.html
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[rustfmt::skip]
enum Mnemonic {
    // Load/Store Operations
    LDA, LDX, LDY, STA, STX, STY,
    // Register Operations
    TAX, TSX, TAY, TXA, TXS, TYA,
    // Stack instructions
    PHA, PHP, PLA, PLP,
    // Logical instructions
    AND, EOR, ORA, BIT,
    // Arithmetic instructions
    ADC, SBC, CMP, CPX, CPY,
    // Increment/Decrement instructions
    INC, INX, INY, DEC, DEX, DEY,
    // Shift instructions
    ASL, LSR, ROL, ROR,
    // Jump instructions
    JMP, JSR, RTS, RTI,
    // Branch instructions
    BCC, BCS, BEQ, BMI, BNE, BPL, BVC, BVS,
    // Flag control instructions
    CLC, CLD, CLI, CLV, SEC, SED, SEI,
    // Misc
    BRK, NOP,
    // Unofficial
    LAX, SAX, DCP, ISB, SLO, RLA, SRE, RRA,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Instruction {
    mnemonic: Mnemonic,
    addressing_mode: AddressingMode,
}

struct CpuBus<T: Bus, U: CpuClock> {
    _bus: std::marker::PhantomData<T>,
    _cycle: std::marker::PhantomData<U>,
}

impl<M: Bus, C: CpuClock> Bus for CpuBus<M, C> {
    fn read(addr: Word, nes: &mut Nes) -> Byte {
        C::tick(nes);
        M::read(addr, nes)
    }

    fn write(addr: Word, value: Byte, nes: &mut Nes) {
        if <Word as Into<u16>>::into(addr) == 0x4014u16 {
            // OAMDMA
            let start: u16 = <Byte as Into<u16>>::into(value) * 0x100u16;
            for a in start..(start + 0xFF) {
                let data = M::read(a.into(), nes);
                C::tick(nes);
                M::write(0x2004u16.into(), data.into(), nes);
                C::tick(nes);
            }
            // dummy cycles
            C::tick(nes);
            if nes.cpu_cycles % 2 == 1 {
                C::tick(nes);
            }
            return;
        }
        C::tick(nes);
        M::write(addr, value, nes);
    }
}

fn push_stack<M: Bus>(value: Byte, nes: &mut Nes) {
    M::write(Word::from(nes.cpu.s) + 0x100, value, nes);
    nes.cpu.s -= 1;
}

fn push_stack_word<M: Bus>(word: Word, nes: &mut Nes) {
    push_stack::<M>((word >> 8).byte(), nes);
    push_stack::<M>((word & 0xFF).byte(), nes);
}

fn pull_stack<M: Bus>(nes: &mut Nes) -> Byte {
    nes.cpu.s += 1;
    M::read(Word::from(nes.cpu.s) + 0x100, nes)
}

fn pull_stack_word<M: Bus>(nes: &mut Nes) -> Word {
    let l: Word = pull_stack::<M>(nes).into();
    let h: Word = pull_stack::<M>(nes).into();
    h << 8 | l
}

fn page_crossed_u16(value: impl Into<u16>, from: impl Into<u16>) -> bool {
    let a = value.into();
    let b = from.into();
    (b.wrapping_add(a) & 0xFF00) != (b & 0xFF00)
}

fn page_crossed(value: impl Into<i64>, from: impl Into<i64>) -> bool {
    let a = value.into();
    let b = from.into();
    (b.wrapping_add(a) & 0xFF00) != (b & 0xFF00)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    enum ClockMock {}
    impl CpuClock for ClockMock {
        fn tick(nes: &mut Nes) {
            nes.cpu_cycles += 1;
        }
    }

    enum BusMock {}
    impl Bus for BusMock {
        fn read(addr: Word, nes: &mut Nes) -> Byte {
            let a: u16 = addr.into();
            nes.wram[a as usize].into()
        }
        fn write(addr: Word, value: Byte, nes: &mut Nes) {
            let a: u16 = addr.into();
            nes.wram[a as usize] = value.into();
        }
    }

    #[test]
    fn test_fetch() {
        let mut nes = Nes::new();

        nes.wram[0x1051] = 0x90;
        nes.wram[0x1052] = 0x3F;
        nes.wram[0x1053] = 0x81;
        nes.wram[0x1054] = 0x90;

        let mut cycles = RefCell::new(0);

        nes.cpu.pc = 0x1052u16.into();

        let instruction = fetch::<BusMock>(&mut nes);
        assert_eq!(instruction, 0x3F.into());

        let instruction = fetch::<BusMock>(&mut nes);
        assert_eq!(instruction, 0x81.into());

        assert_eq!(*(cycles.get_mut()), 2u128);
    }

    #[test]
    fn test_stack() {
        let mut nes = Nes::new();
        nes.cpu.s = 0xFF.into();

        push_stack::<BusMock>(0x83.into(), &mut nes);
        push_stack::<BusMock>(0x14.into(), &mut nes);

        assert_eq!(pull_stack::<BusMock>(&mut nes), 0x14.into());
        assert_eq!(pull_stack::<BusMock>(&mut nes), 0x83.into());
    }

    #[test]
    fn test_stack_word() {
        let mut nes = Nes::new();
        nes.cpu.s = 0xFF.into();

        push_stack_word::<BusMock>(0x98AFu16.into(), &mut nes);
        push_stack_word::<BusMock>(0x003Au16.into(), &mut nes);

        assert_eq!(pull_stack_word::<BusMock>(&mut nes), 0x003Au16.into());
        assert_eq!(pull_stack_word::<BusMock>(&mut nes), 0x98AFu16.into());
    }
}
