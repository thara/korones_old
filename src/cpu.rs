mod instruction_set;
mod interrupt;

use crate::bus::*;
use crate::data_unit::*;

use instruction_set::InstructionSet;

#[derive(Debug, Default, Clone)]
pub struct Cpu {
    a: Byte,
    x: Byte,
    y: Byte,
    s: Byte,
    p: Status,
    pc: Word,

    pub cycles: u128,
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

pub trait CpuStep: Bus {
    fn step(&mut self, cpu: Cpu) -> Cpu {
        let mut cpu = cpu;

        let opcode = self.fetch(&mut cpu);
        let instruction = decode(opcode);
        self.execute(&mut cpu, instruction);
        return cpu;
    }

    fn fetch(&mut self, cpu: &mut Cpu) -> Byte {
        let opcode = self.read(cpu.pc);
        cpu.pc += 1;
        opcode
    }

    // fn get_operand(&mut self, cpu: &mut Cpu, addressing_mode: AddressingMode) -> Operand;
    fn execute(&mut self, cpu: &mut Cpu, instruction: Instruction);
}

pub trait CpuTick {
    fn cpu_tick(&mut self);
}

pub trait CpuStack {
    fn push_stack(&mut self, cpu: &mut Cpu, value: Byte);
    fn push_stack_word(&mut self, cpu: &mut Cpu, word: Word);

    fn pull_stack(&mut self, cpu: &mut Cpu) -> Byte;
    fn pull_stack_word(&mut self, cpu: &mut Cpu) -> Word;
}

impl<T: Bus> CpuStack for T {
    fn push_stack(&mut self, cpu: &mut Cpu, value: Byte) {
        self.write(Word::from(cpu.s) + 0x100, value);
        cpu.s -= 1;
    }

    fn push_stack_word(&mut self, cpu: &mut Cpu, word: Word) {
        self.push_stack(cpu, (word >> 8).byte());
        self.push_stack(cpu, (word & 0xFF).byte());
    }

    fn pull_stack(&mut self, cpu: &mut Cpu) -> Byte {
        cpu.s += 1;
        self.read(Word::from(cpu.s) + 0x100)
    }

    fn pull_stack_word(&mut self, cpu: &mut Cpu) -> Word {
        let l: Word = self.pull_stack(cpu).into();
        let h: Word = self.pull_stack(cpu).into();
        h << 8 | l
    }
}

type Operand = Word;

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
pub struct Instruction {
    mnemonic: Mnemonic,
    addressing_mode: AddressingMode,
}

fn decode(opcode: Byte) -> Instruction {
    let (m, am) = match opcode.u8() {
        0xA9 => (Mnemonic::LDA, AddressingMode::Immediate),
        0xA5 => (Mnemonic::LDA, AddressingMode::ZeroPage),
        0xB5 => (Mnemonic::LDA, AddressingMode::ZeroPageX),
        0xAD => (Mnemonic::LDA, AddressingMode::Absolute),
        0xBD => (Mnemonic::LDA, AddressingMode::AbsoluteX { penalty: true }),
        0xB9 => (Mnemonic::LDA, AddressingMode::AbsoluteY { penalty: true }),
        0xA1 => (Mnemonic::LDA, AddressingMode::IndexedIndirect),
        0xB1 => (Mnemonic::LDA, AddressingMode::IndirectIndexed),
        0xA2 => (Mnemonic::LDX, AddressingMode::Immediate),
        0xA6 => (Mnemonic::LDX, AddressingMode::ZeroPage),
        0xB6 => (Mnemonic::LDX, AddressingMode::ZeroPageY),
        0xAE => (Mnemonic::LDX, AddressingMode::Absolute),
        0xBE => (Mnemonic::LDX, AddressingMode::AbsoluteY { penalty: true }),
        0xA0 => (Mnemonic::LDY, AddressingMode::Immediate),
        0xA4 => (Mnemonic::LDY, AddressingMode::ZeroPage),
        0xB4 => (Mnemonic::LDY, AddressingMode::ZeroPageX),
        0xAC => (Mnemonic::LDY, AddressingMode::Absolute),
        0xBC => (Mnemonic::LDY, AddressingMode::AbsoluteX { penalty: true }),
        0x85 => (Mnemonic::STA, AddressingMode::ZeroPage),
        0x95 => (Mnemonic::STA, AddressingMode::ZeroPageX),
        0x8D => (Mnemonic::STA, AddressingMode::Absolute),
        0x9D => (Mnemonic::STA, AddressingMode::AbsoluteX { penalty: false }),
        0x99 => (Mnemonic::STA, AddressingMode::AbsoluteY { penalty: false }),
        0x81 => (Mnemonic::STA, AddressingMode::IndexedIndirect),
        0x91 => (Mnemonic::STA, AddressingMode::IndirectIndexed),
        0x86 => (Mnemonic::STX, AddressingMode::ZeroPage),
        0x96 => (Mnemonic::STX, AddressingMode::ZeroPageY),
        0x8E => (Mnemonic::STX, AddressingMode::Absolute),
        0x84 => (Mnemonic::STY, AddressingMode::ZeroPage),
        0x94 => (Mnemonic::STY, AddressingMode::ZeroPageX),
        0x8C => (Mnemonic::STY, AddressingMode::Absolute),
        0xAA => (Mnemonic::TAX, AddressingMode::Implicit),
        0xBA => (Mnemonic::TSX, AddressingMode::Implicit),
        0xA8 => (Mnemonic::TAY, AddressingMode::Implicit),
        0x8A => (Mnemonic::TXA, AddressingMode::Implicit),
        0x9A => (Mnemonic::TXS, AddressingMode::Implicit),
        0x98 => (Mnemonic::TYA, AddressingMode::Implicit),

        0x48 => (Mnemonic::PHA, AddressingMode::Implicit),
        0x08 => (Mnemonic::PHP, AddressingMode::Implicit),
        0x68 => (Mnemonic::PLA, AddressingMode::Implicit),
        0x28 => (Mnemonic::PLP, AddressingMode::Implicit),

        0x29 => (Mnemonic::AND, AddressingMode::Immediate),
        0x25 => (Mnemonic::AND, AddressingMode::ZeroPage),
        0x35 => (Mnemonic::AND, AddressingMode::ZeroPageX),
        0x2D => (Mnemonic::AND, AddressingMode::Absolute),
        0x3D => (Mnemonic::AND, AddressingMode::AbsoluteX { penalty: true }),
        0x39 => (Mnemonic::AND, AddressingMode::AbsoluteY { penalty: true }),
        0x21 => (Mnemonic::AND, AddressingMode::IndexedIndirect),
        0x31 => (Mnemonic::AND, AddressingMode::IndirectIndexed),
        0x49 => (Mnemonic::EOR, AddressingMode::Immediate),
        0x45 => (Mnemonic::EOR, AddressingMode::ZeroPage),
        0x55 => (Mnemonic::EOR, AddressingMode::ZeroPageX),
        0x4D => (Mnemonic::EOR, AddressingMode::Absolute),
        0x5D => (Mnemonic::EOR, AddressingMode::AbsoluteX { penalty: true }),
        0x59 => (Mnemonic::EOR, AddressingMode::AbsoluteY { penalty: true }),
        0x41 => (Mnemonic::EOR, AddressingMode::IndexedIndirect),
        0x51 => (Mnemonic::EOR, AddressingMode::IndirectIndexed),
        0x09 => (Mnemonic::ORA, AddressingMode::Immediate),
        0x05 => (Mnemonic::ORA, AddressingMode::ZeroPage),
        0x15 => (Mnemonic::ORA, AddressingMode::ZeroPageX),
        0x0D => (Mnemonic::ORA, AddressingMode::Absolute),
        0x1D => (Mnemonic::ORA, AddressingMode::AbsoluteX { penalty: true }),
        0x19 => (Mnemonic::ORA, AddressingMode::AbsoluteY { penalty: true }),
        0x01 => (Mnemonic::ORA, AddressingMode::IndexedIndirect),
        0x11 => (Mnemonic::ORA, AddressingMode::IndirectIndexed),
        0x24 => (Mnemonic::BIT, AddressingMode::ZeroPage),
        0x2C => (Mnemonic::BIT, AddressingMode::Absolute),

        0x69 => (Mnemonic::ADC, AddressingMode::Immediate),
        0x65 => (Mnemonic::ADC, AddressingMode::ZeroPage),
        0x75 => (Mnemonic::ADC, AddressingMode::ZeroPageX),
        0x6D => (Mnemonic::ADC, AddressingMode::Absolute),
        0x7D => (Mnemonic::ADC, AddressingMode::AbsoluteX { penalty: true }),
        0x79 => (Mnemonic::ADC, AddressingMode::AbsoluteY { penalty: true }),
        0x61 => (Mnemonic::ADC, AddressingMode::IndexedIndirect),
        0x71 => (Mnemonic::ADC, AddressingMode::IndirectIndexed),
        0xE9 => (Mnemonic::SBC, AddressingMode::Immediate),
        0xE5 => (Mnemonic::SBC, AddressingMode::ZeroPage),
        0xF5 => (Mnemonic::SBC, AddressingMode::ZeroPageX),
        0xED => (Mnemonic::SBC, AddressingMode::Absolute),
        0xFD => (Mnemonic::SBC, AddressingMode::AbsoluteX { penalty: true }),
        0xF9 => (Mnemonic::SBC, AddressingMode::AbsoluteY { penalty: true }),
        0xE1 => (Mnemonic::SBC, AddressingMode::IndexedIndirect),
        0xF1 => (Mnemonic::SBC, AddressingMode::IndirectIndexed),
        0xC9 => (Mnemonic::CMP, AddressingMode::Immediate),
        0xC5 => (Mnemonic::CMP, AddressingMode::ZeroPage),
        0xD5 => (Mnemonic::CMP, AddressingMode::ZeroPageX),
        0xCD => (Mnemonic::CMP, AddressingMode::Absolute),
        0xDD => (Mnemonic::CMP, AddressingMode::AbsoluteX { penalty: true }),
        0xD9 => (Mnemonic::CMP, AddressingMode::AbsoluteY { penalty: true }),
        0xC1 => (Mnemonic::CMP, AddressingMode::IndexedIndirect),
        0xD1 => (Mnemonic::CMP, AddressingMode::IndirectIndexed),
        0xE0 => (Mnemonic::CPX, AddressingMode::Immediate),
        0xE4 => (Mnemonic::CPX, AddressingMode::ZeroPage),
        0xEC => (Mnemonic::CPX, AddressingMode::Absolute),
        0xC0 => (Mnemonic::CPY, AddressingMode::Immediate),
        0xC4 => (Mnemonic::CPY, AddressingMode::ZeroPage),
        0xCC => (Mnemonic::CPY, AddressingMode::Absolute),

        0xE6 => (Mnemonic::INC, AddressingMode::ZeroPage),
        0xF6 => (Mnemonic::INC, AddressingMode::ZeroPageX),
        0xEE => (Mnemonic::INC, AddressingMode::Absolute),
        0xFE => (Mnemonic::INC, AddressingMode::AbsoluteX { penalty: false }),
        0xE8 => (Mnemonic::INX, AddressingMode::Implicit),
        0xC8 => (Mnemonic::INY, AddressingMode::Implicit),
        0xC6 => (Mnemonic::DEC, AddressingMode::ZeroPage),
        0xD6 => (Mnemonic::DEC, AddressingMode::ZeroPageX),
        0xCE => (Mnemonic::DEC, AddressingMode::Absolute),
        0xDE => (Mnemonic::DEC, AddressingMode::AbsoluteX { penalty: false }),
        0xCA => (Mnemonic::DEX, AddressingMode::Implicit),
        0x88 => (Mnemonic::DEY, AddressingMode::Implicit),

        0x0A => (Mnemonic::ASL, AddressingMode::Accumulator),
        0x06 => (Mnemonic::ASL, AddressingMode::ZeroPage),
        0x16 => (Mnemonic::ASL, AddressingMode::ZeroPageX),
        0x0E => (Mnemonic::ASL, AddressingMode::Absolute),
        0x1E => (Mnemonic::ASL, AddressingMode::AbsoluteX { penalty: false }),
        0x4A => (Mnemonic::LSR, AddressingMode::Accumulator),
        0x46 => (Mnemonic::LSR, AddressingMode::ZeroPage),
        0x56 => (Mnemonic::LSR, AddressingMode::ZeroPageX),
        0x4E => (Mnemonic::LSR, AddressingMode::Absolute),
        0x5E => (Mnemonic::LSR, AddressingMode::AbsoluteX { penalty: false }),
        0x2A => (Mnemonic::ROL, AddressingMode::Accumulator),
        0x26 => (Mnemonic::ROL, AddressingMode::ZeroPage),
        0x36 => (Mnemonic::ROL, AddressingMode::ZeroPageX),
        0x2E => (Mnemonic::ROL, AddressingMode::Absolute),
        0x3E => (Mnemonic::ROL, AddressingMode::AbsoluteX { penalty: false }),
        0x6A => (Mnemonic::ROR, AddressingMode::Accumulator),
        0x66 => (Mnemonic::ROR, AddressingMode::ZeroPage),
        0x76 => (Mnemonic::ROR, AddressingMode::ZeroPageX),
        0x6E => (Mnemonic::ROR, AddressingMode::Absolute),
        0x7E => (Mnemonic::ROR, AddressingMode::AbsoluteX { penalty: false }),

        0x4C => (Mnemonic::JMP, AddressingMode::Absolute),
        0x6C => (Mnemonic::JMP, AddressingMode::Indirect),
        0x20 => (Mnemonic::JSR, AddressingMode::Absolute),
        0x60 => (Mnemonic::RTS, AddressingMode::Implicit),
        0x40 => (Mnemonic::RTI, AddressingMode::Implicit),

        0x90 => (Mnemonic::BCC, AddressingMode::Relative),
        0xB0 => (Mnemonic::BCS, AddressingMode::Relative),
        0xF0 => (Mnemonic::BEQ, AddressingMode::Relative),
        0x30 => (Mnemonic::BMI, AddressingMode::Relative),
        0xD0 => (Mnemonic::BNE, AddressingMode::Relative),
        0x10 => (Mnemonic::BPL, AddressingMode::Relative),
        0x50 => (Mnemonic::BVC, AddressingMode::Relative),
        0x70 => (Mnemonic::BVS, AddressingMode::Relative),

        0x18 => (Mnemonic::CLC, AddressingMode::Implicit),
        0xD8 => (Mnemonic::CLD, AddressingMode::Implicit),
        0x58 => (Mnemonic::CLI, AddressingMode::Implicit),
        0xB8 => (Mnemonic::CLV, AddressingMode::Implicit),

        0x38 => (Mnemonic::SEC, AddressingMode::Implicit),
        0xF8 => (Mnemonic::SED, AddressingMode::Implicit),
        0x78 => (Mnemonic::SEI, AddressingMode::Implicit),

        0x00 => (Mnemonic::BRK, AddressingMode::Implicit),

        // Undocumented
        0xEB => (Mnemonic::SBC, AddressingMode::Immediate),

        0x04 | 0x44 | 0x64 => (Mnemonic::NOP, AddressingMode::ZeroPage),
        0x0C => (Mnemonic::NOP, AddressingMode::Absolute),
        0x14 | 0x34 | 0x54 | 0x74 | 0xD4 | 0xF4 => (Mnemonic::NOP, AddressingMode::ZeroPageX),
        0x1A | 0x3A | 0x5A | 0x7A | 0xDA | 0xEA | 0xFA => (Mnemonic::NOP, AddressingMode::Implicit),
        0x1C | 0x3C | 0x5C | 0x7C | 0xDC | 0xFC => {
            (Mnemonic::NOP, AddressingMode::AbsoluteX { penalty: true })
        }
        0x80 | 0x82 | 0x89 | 0xC2 | 0xE2 => (Mnemonic::NOP, AddressingMode::Immediate),

        0xA3 => (Mnemonic::LAX, AddressingMode::IndexedIndirect),
        0xA7 => (Mnemonic::LAX, AddressingMode::ZeroPage),
        0xAF => (Mnemonic::LAX, AddressingMode::Absolute),
        0xB3 => (Mnemonic::LAX, AddressingMode::IndirectIndexed),
        0xB7 => (Mnemonic::LAX, AddressingMode::ZeroPageY),
        0xBF => (Mnemonic::LAX, AddressingMode::AbsoluteY { penalty: true }),

        0x83 => (Mnemonic::SAX, AddressingMode::IndexedIndirect),
        0x87 => (Mnemonic::SAX, AddressingMode::ZeroPage),
        0x8F => (Mnemonic::SAX, AddressingMode::Absolute),
        0x97 => (Mnemonic::SAX, AddressingMode::ZeroPageY),

        0xC3 => (Mnemonic::DCP, AddressingMode::IndexedIndirect),
        0xC7 => (Mnemonic::DCP, AddressingMode::ZeroPage),
        0xCF => (Mnemonic::DCP, AddressingMode::Absolute),
        0xD3 => (Mnemonic::DCP, AddressingMode::IndirectIndexed),
        0xD7 => (Mnemonic::DCP, AddressingMode::ZeroPageX),
        0xDB => (Mnemonic::DCP, AddressingMode::AbsoluteY { penalty: false }),
        0xDF => (Mnemonic::DCP, AddressingMode::AbsoluteX { penalty: false }),

        0xE3 => (Mnemonic::ISB, AddressingMode::IndexedIndirect),
        0xE7 => (Mnemonic::ISB, AddressingMode::ZeroPage),
        0xEF => (Mnemonic::ISB, AddressingMode::Absolute),
        0xF3 => (Mnemonic::ISB, AddressingMode::IndirectIndexed),
        0xF7 => (Mnemonic::ISB, AddressingMode::ZeroPageX),
        0xFB => (Mnemonic::ISB, AddressingMode::AbsoluteY { penalty: false }),
        0xFF => (Mnemonic::ISB, AddressingMode::AbsoluteX { penalty: false }),

        0x03 => (Mnemonic::SLO, AddressingMode::IndexedIndirect),
        0x07 => (Mnemonic::SLO, AddressingMode::ZeroPage),
        0x0F => (Mnemonic::SLO, AddressingMode::Absolute),
        0x13 => (Mnemonic::SLO, AddressingMode::IndirectIndexed),
        0x17 => (Mnemonic::SLO, AddressingMode::ZeroPageX),
        0x1B => (Mnemonic::SLO, AddressingMode::AbsoluteY { penalty: false }),
        0x1F => (Mnemonic::SLO, AddressingMode::AbsoluteX { penalty: false }),

        0x23 => (Mnemonic::RLA, AddressingMode::IndexedIndirect),
        0x27 => (Mnemonic::RLA, AddressingMode::ZeroPage),
        0x2F => (Mnemonic::RLA, AddressingMode::Absolute),
        0x33 => (Mnemonic::RLA, AddressingMode::IndirectIndexed),
        0x37 => (Mnemonic::RLA, AddressingMode::ZeroPageX),
        0x3B => (Mnemonic::RLA, AddressingMode::AbsoluteY { penalty: false }),
        0x3F => (Mnemonic::RLA, AddressingMode::AbsoluteX { penalty: false }),

        0x43 => (Mnemonic::SRE, AddressingMode::IndexedIndirect),
        0x47 => (Mnemonic::SRE, AddressingMode::ZeroPage),
        0x4F => (Mnemonic::SRE, AddressingMode::Absolute),
        0x53 => (Mnemonic::SRE, AddressingMode::IndirectIndexed),
        0x57 => (Mnemonic::SRE, AddressingMode::ZeroPageX),
        0x5B => (Mnemonic::SRE, AddressingMode::AbsoluteY { penalty: false }),
        0x5F => (Mnemonic::SRE, AddressingMode::AbsoluteX { penalty: false }),

        0x63 => (Mnemonic::RRA, AddressingMode::IndexedIndirect),
        0x67 => (Mnemonic::RRA, AddressingMode::ZeroPage),
        0x6F => (Mnemonic::RRA, AddressingMode::Absolute),
        0x73 => (Mnemonic::RRA, AddressingMode::IndirectIndexed),
        0x77 => (Mnemonic::RRA, AddressingMode::ZeroPageX),
        0x7B => (Mnemonic::RRA, AddressingMode::AbsoluteY { penalty: false }),
        0x7F => (Mnemonic::RRA, AddressingMode::AbsoluteX { penalty: false }),

        _ => (Mnemonic::NOP, AddressingMode::Implicit),
    };
    Instruction {
        mnemonic: m,
        addressing_mode: am,
    }
}

trait GetOperand: CpuTick + Bus {
    fn get_operand(&mut self, cpu: &mut Cpu, addressing_mode: AddressingMode) -> Operand {
        match addressing_mode {
            AddressingMode::Implicit => Word::from(0x00u16),
            AddressingMode::Accumulator => cpu.a.into(),
            AddressingMode::Immediate => {
                let operand = cpu.pc;
                cpu.pc += 1;
                operand
            }
            AddressingMode::ZeroPage => {
                let operand = Word::from(self.read(cpu.pc)) & 0xFF;
                cpu.pc += 1;
                operand
            }
            AddressingMode::ZeroPageX => {
                let operand = (Word::from(self.read(cpu.pc)) + Word::from(cpu.x)) & 0xFF;
                cpu.pc += 1;
                self.cpu_tick();
                operand
            }
            AddressingMode::ZeroPageY => {
                let operand = (Word::from(self.read(cpu.pc)) + Word::from(cpu.y)) & 0xFF;
                cpu.pc += 1;
                self.cpu_tick();
                operand
            }
            AddressingMode::Absolute => {
                let operand = self.read_word(cpu.pc);
                cpu.pc += 2;
                operand
            }
            AddressingMode::AbsoluteX { penalty } => {
                let data = self.read_word(cpu.pc);
                let operand = data + Word::from(cpu.x);
                cpu.pc += 2;
                if penalty {
                    if page_crossed_u16(cpu.x, data) {
                        self.cpu_tick();
                    }
                } else {
                    self.cpu_tick();
                }
                operand
            }
            AddressingMode::AbsoluteY { penalty } => {
                let data = self.read_word(cpu.pc);
                let operand = data + Word::from(cpu.y);
                cpu.pc += 2;
                if penalty {
                    if page_crossed_u16(cpu.y, data) {
                        self.cpu_tick();
                    }
                } else {
                    self.cpu_tick();
                }
                operand
            }
            AddressingMode::Relative => {
                let operand: Word = self.read(cpu.pc).into();
                cpu.pc += 1;
                operand
            }
            AddressingMode::Indirect => {
                let data = self.read_word(cpu.pc);
                let operand = self.read_on_indirect(data);
                cpu.pc += 2;
                operand
            }
            AddressingMode::IndexedIndirect => {
                let data = self.read(cpu.pc);
                let operand = self.read_on_indirect(Word::from(data + cpu.x) & 0xFF);
                cpu.pc += 1;
                self.cpu_tick();
                operand
            }
            AddressingMode::IndirectIndexed => {
                let y: Word = cpu.y.into();
                let data: Word = self.read(cpu.pc).into();
                let operand = self.read_on_indirect(data) + y;
                cpu.pc += 1;
                if page_crossed_u16(y, operand - y) {
                    self.cpu_tick();
                }
                operand
            }
        }
    }
}

impl<T: CpuTick + Bus> GetOperand for T {}
impl<T: CpuTick + Bus + CpuStack> InstructionSet for T {}

impl<T: CpuTick + Bus + CpuStack> CpuStep for T {
    fn execute(&mut self, cpu: &mut Cpu, instruction: Instruction) {
        let operand = self.get_operand(cpu, instruction.addressing_mode);

        match (instruction.mnemonic, instruction.addressing_mode) {
            (Mnemonic::LDA, _) => self.lda(cpu, operand),
            (Mnemonic::LDX, _) => self.ldx(cpu, operand),
            (Mnemonic::LDY, _) => self.ldy(cpu, operand),
            (Mnemonic::STA, AddressingMode::IndirectIndexed) => {
                self.sta(cpu, operand);
                self.cpu_tick();
            }
            (Mnemonic::STA, _) => self.sta(cpu, operand),
            (Mnemonic::STX, _) => self.stx(cpu, operand),
            (Mnemonic::STY, _) => self.sty(cpu, operand),
            (Mnemonic::TAX, _) => self.tax(cpu),
            (Mnemonic::TSX, _) => self.tsx(cpu),
            (Mnemonic::TAY, _) => self.tay(cpu),
            (Mnemonic::TXA, _) => self.txa(cpu),
            (Mnemonic::TXS, _) => self.txs(cpu),
            (Mnemonic::TYA, _) => self.tya(cpu),
            (Mnemonic::PHA, _) => self.pha(cpu),
            (Mnemonic::PHP, _) => self.php(cpu),
            (Mnemonic::PLA, _) => self.pla(cpu),
            (Mnemonic::PLP, _) => self.plp(cpu),
            (Mnemonic::AND, _) => self.and(cpu, operand),
            (Mnemonic::EOR, _) => self.eor(cpu, operand),
            (Mnemonic::ORA, _) => self.ora(cpu, operand),
            (Mnemonic::BIT, _) => self.bit(cpu, operand),
            (Mnemonic::ADC, _) => self.adc(cpu, operand),
            (Mnemonic::SBC, _) => self.sbc(cpu, operand),
            (Mnemonic::CMP, _) => self.cmp(cpu, operand),
            (Mnemonic::CPX, _) => self.cpx(cpu, operand),
            (Mnemonic::CPY, _) => self.cpy(cpu, operand),
            (Mnemonic::INC, _) => self.inc(cpu, operand),
            (Mnemonic::INX, _) => self.inx(cpu),
            (Mnemonic::INY, _) => self.iny(cpu),
            (Mnemonic::DEC, _) => self.dec(cpu, operand),
            (Mnemonic::DEX, _) => self.dex(cpu),
            (Mnemonic::DEY, _) => self.dey(cpu),
            (Mnemonic::ASL, AddressingMode::Accumulator) => self.asl_for_accumelator(cpu),
            (Mnemonic::ASL, _) => self.asl(cpu, operand),
            (Mnemonic::LSR, AddressingMode::Accumulator) => self.lsr_for_accumelator(cpu),
            (Mnemonic::LSR, _) => self.lsr(cpu, operand),
            (Mnemonic::ROL, AddressingMode::Accumulator) => self.rol_for_accumelator(cpu),
            (Mnemonic::ROL, _) => self.rol(cpu, operand),
            (Mnemonic::ROR, AddressingMode::Accumulator) => self.ror_for_accumelator(cpu),
            (Mnemonic::ROR, _) => self.ror(cpu, operand),
            (Mnemonic::JMP, _) => self.jmp(cpu, operand),
            (Mnemonic::JSR, _) => self.jsr(cpu, operand),
            (Mnemonic::RTS, _) => self.rts(cpu),
            (Mnemonic::RTI, _) => self.rti(cpu),
            (Mnemonic::BCC, _) => self.bcc(cpu, operand),
            (Mnemonic::BCS, _) => self.bcs(cpu, operand),
            (Mnemonic::BEQ, _) => self.beq(cpu, operand),
            (Mnemonic::BMI, _) => self.bmi(cpu, operand),
            (Mnemonic::BNE, _) => self.bne(cpu, operand),
            (Mnemonic::BPL, _) => self.bpl(cpu, operand),
            (Mnemonic::BVC, _) => self.bvc(cpu, operand),
            (Mnemonic::BVS, _) => self.bvs(cpu, operand),
            (Mnemonic::CLC, _) => self.clc(cpu),
            (Mnemonic::CLD, _) => self.cld(cpu),
            (Mnemonic::CLI, _) => self.cli(cpu),
            (Mnemonic::CLV, _) => self.clv(cpu),
            (Mnemonic::SEC, _) => self.sec(cpu),
            (Mnemonic::SED, _) => self.sed(cpu),
            (Mnemonic::SEI, _) => self.sei(cpu),
            (Mnemonic::BRK, _) => self.brk(cpu),
            (Mnemonic::NOP, _) => self.nop(cpu),
            (Mnemonic::LAX, _) => self.lax(cpu, operand),
            (Mnemonic::SAX, _) => self.sax(cpu, operand),
            (Mnemonic::DCP, _) => self.dcp(cpu, operand),
            (Mnemonic::ISB, _) => self.isb(cpu, operand),
            (Mnemonic::SLO, _) => self.slo(cpu, operand),
            (Mnemonic::RLA, _) => self.rla(cpu, operand),
            (Mnemonic::SRE, _) => self.sre(cpu, operand),
            (Mnemonic::RRA, _) => self.rra(cpu, operand),
        }
    }
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

    struct TestNes {
        wram: [u8; 0x2000],
        cycles: u64,
    }

    impl Bus for TestNes {
        fn read(&mut self, addr: impl Into<Word>) -> Byte {
            let w = addr.into();
            let a: u16 = w.into();
            self.wram[a as usize].into()
        }
        fn write(&mut self, addr: impl Into<Word>, value: impl Into<Byte>) {
            let w = addr.into();
            let a: u16 = w.into();
            let v = value.into();
            self.wram[a as usize] = v.into();
        }
    }

    impl CpuTick for TestNes {
        fn cpu_tick(&mut self) {
            self.cycles += 1;
        }
    }

    #[test]
    fn test_fetch() {
        let mut nes = TestNes {
            wram: [0; 0x2000],
            cycles: 0,
        };

        nes.wram[0x1051] = 0x90;
        nes.wram[0x1052] = 0x3F;
        nes.wram[0x1053] = 0x81;
        nes.wram[0x1054] = 0x90;

        let mut cpu = Cpu::default();
        cpu.pc = 0x1052u16.into();

        let instruction = nes.fetch(&mut cpu);
        assert_eq!(instruction, 0x3F.into());

        let instruction = nes.fetch(&mut cpu);
        assert_eq!(instruction, 0x81.into());
    }

    #[test]
    fn test_stack() {
        let mut nes = TestNes {
            wram: [0; 0x2000],
            cycles: 0,
        };
        let mut cpu = Cpu::default();

        cpu.s = 0xFF.into();

        nes.push_stack(&mut cpu, 0x83.into());
        nes.push_stack(&mut cpu, 0x14.into());

        assert_eq!(nes.pull_stack(&mut cpu), 0x14.into());
        assert_eq!(nes.pull_stack(&mut cpu), 0x83.into());
    }

    #[test]
    fn test_stack_word() {
        let mut nes = TestNes {
            wram: [0; 0x2000],
            cycles: 0,
        };
        let mut cpu = Cpu::default();

        cpu.s = 0xFF.into();

        nes.push_stack_word(&mut cpu, 0x98AFu16.into());
        nes.push_stack_word(&mut cpu, 0x003Au16.into());

        assert_eq!(nes.pull_stack_word(&mut cpu), 0x003Au16.into());
        assert_eq!(nes.pull_stack_word(&mut cpu), 0x98AFu16.into());
    }
}
