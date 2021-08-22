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
    fn read_on_indirect(addr: Word, nes: &mut Nes) -> Word {
        let low = Self::read(addr, nes);
        // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
        let high = Self::read(addr & 0xFF00 | ((addr + 1) & 0x00FF), nes);
        Word::from(low) | (Word::from(high) << 8)
    }
}
