use crate::data_unit::*;

pub trait Bus {
    fn read(&mut self, addr: impl Into<Word>) -> Byte;
    fn write(&mut self, addr: impl Into<Word>, value: impl Into<Byte>);

    fn read_word(&mut self, addr: Word) -> Word {
        Word::from(self.read(addr)) | (Word::from(self.read(addr + 1)) << 8)
    }

    fn read_on_indirect(&mut self, addr: Word) -> Word {
        let low = self.read(addr);
        // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
        let high = self.read(addr & 0xFF00 | ((addr + 1) & 0x00FF));
        Word::from(low) | (Word::from(high) << 8)
    }
}
