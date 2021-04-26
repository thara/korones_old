use super::*;

use crate::bus::ReadWord;
use crate::prelude::*;

pub(super) fn reset<M: Bus>(nes: &mut Nes) {
    nes.cpu_cycles += 5;
    nes.cpu.pc = M::read_word(0xFFFCu16.into(), nes);
    nes.cpu.p.insert(Status::I);
    nes.cpu.s -= 3;
}

// NMI
pub(super) fn non_markable_interrupt<M: Bus>(nes: &mut Nes) {
    nes.cycles += 2;
    push_stack_word::<M>(nes.cpu.pc, nes);
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    push_stack::<M>((nes.cpu.p | Status::INTERRUPTED_B).bits().into(), nes);
    nes.cpu.p.insert(Status::I);
    nes.cpu.pc = M::read_word(0xFFFAu16.into(), nes)
}

// IRQ
pub(super) fn interrupt_request<M: Bus>(nes: &mut Nes) {
    nes.cycles += 2;
    push_stack_word::<M>(nes.cpu.pc, nes);
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    push_stack::<M>((nes.cpu.p | Status::INTERRUPTED_B).bits().into(), nes);
    nes.cpu.p.insert(Status::I);
    nes.cpu.pc = M::read_word(0xFFFEu16.into(), nes)
}

// BRK
pub(super) fn break_interrupt<M: Bus>(nes: &mut Nes) {
    nes.cycles += 2;
    nes.cpu.pc += 1;
    push_stack_word::<M>(nes.cpu.pc, nes);
    // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
    // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
    push_stack::<M>((nes.cpu.p | Status::INTERRUPTED_B).bits().into(), nes);
    nes.cpu.p.insert(Status::I);
    nes.cpu.pc = M::read_word(0xFFFEu16.into(), nes)
}
