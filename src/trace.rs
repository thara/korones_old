use std::fmt;

use crate::cpu::*;
use crate::data_unit::*;
use crate::nes::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Trace {
    pc: Word,
    operation: Byte,
    operand_1: Byte,
    operand_2: Byte,
    a: Byte,
    x: Byte,
    y: Byte,
    sp: Byte,
    p: Byte,
    cycle: u128,

    mnemonic: Mnemonic,
    addressing_mode: AddressingMode,
    assembly_code: String,
}

impl Trace {
    pub fn new(nes: &mut Nes) -> Self {
        let instruction = nes.read_bus(nes.cpu.pc);
        let (mnemonic, addressing_mode) = decode(instruction);
        let assembly_code = to_assembly_code(instruction, mnemonic, addressing_mode, nes);
        Self {
            pc: nes.cpu.pc,
            operation: nes.read_bus(nes.cpu.pc),
            operand_1: nes.read_bus(nes.cpu.pc + 1),
            operand_2: nes.read_bus(nes.cpu.pc + 2),
            a: nes.cpu.a,
            x: nes.cpu.x,
            y: nes.cpu.y,
            sp: nes.cpu.s,
            p: nes.cpu.p.bits().into(),
            cycle: nes.cpu.cycles,
            mnemonic,
            addressing_mode,
            assembly_code,
        }
    }
}

impl fmt::Display for Trace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let len = self.addressing_mode.instruction_length();
        let machine_code = match len {
            3 => format!(
                "{:02X} {:02X} {:02X}",
                self.operation, self.operand_1, self.operand_2
            ),
            2 => format!("{:02X} {:02X}   ", self.operation, self.operand_1),
            _ => format!("{:02X}      ", self.operation),
        };
        write!(
            f,
            "{:04X}  {} {}A:{:02X} X:{:02X} Y:{:02X} P:{:02X} SP:{:02X} CYC:{}",
            self.pc,
            machine_code,
            self.assembly_code,
            self.a,
            self.x,
            self.y,
            self.p,
            self.sp,
            self.cycle
        )
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl fmt::UpperHex for Byte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = <Self as Into<u8>>::into(*self);
        fmt::UpperHex::fmt(&v, f)
    }
}

impl fmt::UpperHex for Word {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let v = <Self as Into<u16>>::into(*self);
        fmt::UpperHex::fmt(&v, f)
    }
}

fn to_assembly_code(
    operation: Byte,
    mnemonic: Mnemonic,
    addressing_mode: AddressingMode,
    nes: &mut Nes,
) -> String {
    let name = mnemonic.to_string();
    let prefix = if UNDOCUMENTED_OPCODES.contains(&operation.u8()) {
        "*"
    } else {
        " "
    };

    let operand = match (mnemonic, addressing_mode) {
        (Mnemonic::JMP, AddressingMode::Absolute) | (Mnemonic::JSR, AddressingMode::Absolute) => {
            format!("${:4X}", decode_address(addressing_mode, nes))
        }
        (Mnemonic::LSR, AddressingMode::Accumulator)
        | (Mnemonic::ASL, AddressingMode::Accumulator)
        | (Mnemonic::ROR, AddressingMode::Accumulator)
        | (Mnemonic::ROL, AddressingMode::Accumulator) => "A".to_string(),

        (_, addressing_mode) => match addressing_mode {
            AddressingMode::Implicit | AddressingMode::Accumulator => " ".to_string(),
            AddressingMode::Immediate => format!("#${:02X}", cpu_operand_1(nes)),
            AddressingMode::ZeroPage => format!(
                "${:02X} = {:02X}",
                cpu_operand_1(nes),
                read(addressing_mode, nes)
            ),
            AddressingMode::ZeroPageX => format!(
                "${:02X},X @ {:02X} = {:02X}",
                cpu_operand_1(nes),
                cpu_operand_1(nes) + nes.cpu.x,
                read(addressing_mode, nes)
            ),
            AddressingMode::ZeroPageY => format!(
                "${:02X},Y @ {:02X} = {:02X}",
                cpu_operand_1(nes),
                cpu_operand_1(nes) + nes.cpu.y,
                read(addressing_mode, nes)
            ),
            AddressingMode::Absolute => format!(
                "${:04X} = {:02X}",
                cpu_operand_16(nes),
                read(addressing_mode, nes)
            ),
            AddressingMode::AbsoluteX { .. } => format!(
                "${:04X},X @ {:04X} = {:02X}",
                cpu_operand_16(nes),
                cpu_operand_16(nes) + nes.cpu.x,
                read(addressing_mode, nes)
            ),
            AddressingMode::AbsoluteY { .. } => format!(
                "${:04X},Y @ {:04X} = {:02X}",
                cpu_operand_16(nes),
                cpu_operand_16(nes) + nes.cpu.y,
                read(addressing_mode, nes)
            ),
            AddressingMode::Relative => {
                let pc = <Word as Into<i16>>::into(nes.cpu.pc);
                let offset = <Byte as Into<i8>>::into(cpu_operand_1(nes));
                format!("${:04X}", pc.wrapping_add(2).wrapping_add(offset as i16))
            }
            AddressingMode::Indirect => format!(
                "(${:04X}) = {:04X}",
                cpu_operand_16(nes),
                read_on_indirect(cpu_operand_16(nes), nes)
            ),
            AddressingMode::IndexedIndirect => {
                let operand_x = cpu_operand_1(nes) + nes.cpu.x;
                let addr = read_on_indirect(operand_x.into(), nes);
                format!(
                    "(${:02X},X) @ {:02X} = {:04X} = {:02X}",
                    cpu_operand_1(nes),
                    operand_x,
                    addr,
                    nes.read_bus(addr)
                )
            }
            AddressingMode::IndirectIndexed => {
                let addr = read_on_indirect(cpu_operand_1(nes).into(), nes);
                format!(
                    "(${:02X}),Y = {:04X} @ {:04X} = {:02X}",
                    cpu_operand_1(nes),
                    addr,
                    addr + nes.cpu.y,
                    nes.read_bus(addr + nes.cpu.y)
                )
            }
        },
    };
    format!("{}{} {:<28}", prefix, name, operand)
}

fn read(addressing_mode: AddressingMode, nes: &mut Nes) -> Byte {
    let addr = decode_address(addressing_mode, nes);
    let addr: u16 = addr.into();
    match addr {
        // APU status always returns 0xFF
        // http://archive.nes.science/nesdev-forums/f3/t17748.xhtml
        0x4004..=0x4007 | 0x4015 => 0xFF.into(),
        _ => nes.read_bus(addr),
    }
}

fn decode_address(addressing_mode: AddressingMode, nes: &mut Nes) -> Word {
    match addressing_mode {
        AddressingMode::Implicit => 0x00u16.into(),
        AddressingMode::Immediate => nes.cpu.pc,
        AddressingMode::ZeroPage => cpu_operand_1(nes).into(),
        AddressingMode::ZeroPageX => {
            <Byte as Into<Word>>::into(cpu_operand_1(nes) + nes.cpu.x) & 0xFF
        }
        AddressingMode::ZeroPageY => {
            <Byte as Into<Word>>::into(cpu_operand_1(nes) + nes.cpu.y) & 0xFF
        }
        AddressingMode::Absolute => cpu_operand_16(nes),
        AddressingMode::AbsoluteX { .. } => cpu_operand_16(nes) + nes.cpu.x,
        AddressingMode::AbsoluteY { .. } => cpu_operand_16(nes) + nes.cpu.y,
        AddressingMode::Relative => nes.cpu.pc,
        AddressingMode::Indirect => read_on_indirect(cpu_operand_16(nes), nes),
        AddressingMode::IndexedIndirect => {
            read_on_indirect((cpu_operand_16(nes) + nes.cpu.x) & 0xFF, nes)
        }
        AddressingMode::IndirectIndexed => read_on_indirect(cpu_operand_16(nes), nes) + nes.cpu.y,
        _ => 0x00u16.into(),
    }
}

fn cpu_operand_1(nes: &mut Nes) -> Byte {
    nes.read_bus(nes.cpu.pc + 1)
}

fn cpu_operand_2(nes: &mut Nes) -> Byte {
    nes.read_bus(nes.cpu.pc + 2)
}

fn cpu_operand_16(nes: &mut Nes) -> Word {
    <Byte as Into<Word>>::into(cpu_operand_1(nes))
        | <Byte as Into<Word>>::into(cpu_operand_2(nes)) << 8
}

fn read_on_indirect(addr: Word, nes: &mut Nes) -> Word {
    let low = nes.read_bus(addr);
    // Reproduce 6502 bug; http://nesdev.com/6502bugs.txt
    let high = nes.read_bus(addr & 0xFF00 | ((addr + 1) & 0x00FF));
    Word::from(low) | (Word::from(high) << 8)
}

impl AddressingMode {
    fn instruction_length(&self) -> u8 {
        match self {
            Self::Immediate
            | Self::ZeroPage
            | Self::ZeroPageX
            | Self::ZeroPageY
            | Self::Relative
            | Self::IndirectIndexed
            | Self::IndexedIndirect => 2,
            Self::Indirect | Self::Absolute | Self::AbsoluteX { .. } | Self::AbsoluteY { .. } => 3,
            _ => 1,
        }
    }
}

const UNDOCUMENTED_OPCODES: [u8; 80] = [
    0xEB, 0x04, 0x44, 0x64, 0x0C, 0x14, 0x34, 0x54, 0x74, 0xD4, 0xF4, 0x1A, 0x3A, 0x5A, 0x7A, 0xDA,
    0xFA, 0x1C, 0x3C, 0x5C, 0x7C, 0xDC, 0xFC, 0x80, 0x82, 0x89, 0xC2, 0xE2, 0xA3, 0xA7, 0xAF, 0xB3,
    0xB7, 0xBF, 0x83, 0x87, 0x8F, 0x97, 0xC3, 0xC7, 0xCF, 0xD3, 0xD7, 0xDB, 0xDF, 0xE3, 0xE7, 0xEF,
    0xF3, 0xF7, 0xFB, 0xFF, 0x03, 0x07, 0x0F, 0x13, 0x17, 0x1B, 0x1F, 0x23, 0x27, 0x2F, 0x33, 0x37,
    0x3B, 0x3F, 0x43, 0x47, 0x4F, 0x53, 0x57, 0x5B, 0x5F, 0x63, 0x67, 0x6F, 0x73, 0x77, 0x7B, 0x7F,
];
