use crate::prelude::*;

pub trait Bus {
    fn read(addr: Word, from: &mut Nes) -> Byte;
    fn write(addr: Word, value: Byte, to: &mut Nes);
}

pub trait ReadWord: Bus {
    fn read_word(addr: Word, from: &mut Nes) -> Word;
}

impl<T> ReadWord for T
where
    T: Bus,
{
    fn read_word(addr: Word, from: &mut Nes) -> Word {
        Word::from(Self::read(addr, from)) | (Word::from(Self::read(addr + 1, from)) << 8)
    }
}

pub trait ReadOnIndirect: Bus {
    fn read_on_indirect(operand: Word, from: &mut Nes) -> Word;
}

impl<T> ReadOnIndirect for T
where
    T: Bus,
{
    fn read_on_indirect(operand: Word, from: &mut Nes) -> Word {
        let low = Word::from(Self::read(operand, from));
        // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
        let addr = operand & 0xFF00 | ((operand + 1) & 0x00FF);
        let high = Word::from(Self::read(addr, from)) << 8;
        low | high
    }
}
