use super::*;
use crate::bus::*;
use crate::data_unit::*;

pub(super) trait InstructionSet: CpuTick + Bus + CpuStack {
    // LoaD Accumulator
    fn lda(&mut self, cpu: &mut Cpu, operand: Operand) {
        cpu.a = self.read(operand);
        cpu.p.set_zn(cpu.a)
    }

    // LoaD X register
    fn ldx(&mut self, cpu: &mut Cpu, operand: Operand) {
        cpu.x = self.read(operand);
        cpu.p.set_zn(cpu.x)
    }

    // LoaD Y register
    fn ldy(&mut self, cpu: &mut Cpu, operand: Operand) {
        cpu.y = self.read(operand);
        cpu.p.set_zn(cpu.y)
    }

    // STore Accumulator
    fn sta(&mut self, cpu: &mut Cpu, operand: Operand) {
        self.write(operand, cpu.a)
    }

    // STore X register
    fn stx(&mut self, cpu: &mut Cpu, operand: Operand) {
        self.write(operand, cpu.x)
    }

    // STore Y register
    fn sty(&mut self, cpu: &mut Cpu, operand: Operand) {
        self.write(operand, cpu.y)
    }

    // Transfer Accumulator to X
    fn tax(&mut self, cpu: &mut Cpu) {
        cpu.x = cpu.a;
        cpu.p.set_zn(cpu.x);
        self.cpu_tick();
    }

    // Transfer Stack pointer to X
    fn tsx(&mut self, cpu: &mut Cpu) {
        cpu.x = cpu.s;
        cpu.p.set_zn(cpu.x);
        self.cpu_tick();
    }

    // Transfer Accumulator to Y
    fn tay(&mut self, cpu: &mut Cpu) {
        cpu.y = cpu.a;
        cpu.p.set_zn(cpu.y);
        self.cpu_tick();
    }

    // Transfer X to Accumulator
    fn txa(&mut self, cpu: &mut Cpu) {
        cpu.a = cpu.x;
        cpu.p.set_zn(cpu.a);
        self.cpu_tick();
    }

    // Transfer X to Stack pointer
    fn txs(&mut self, cpu: &mut Cpu) {
        cpu.s = cpu.x;
        self.cpu_tick();
    }

    // Transfer Y to Accumulator
    fn tya(&mut self, cpu: &mut Cpu) {
        cpu.a = cpu.y;
        cpu.p.set_zn(cpu.a);
        self.cpu_tick();
    }

    // PusH Accumulator
    fn pha(&mut self, cpu: &mut Cpu) {
        self.push_stack(cpu, cpu.a);
        self.cpu_tick();
    }

    // PusH Processor status
    fn php(&mut self, cpu: &mut Cpu) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(cpu, (cpu.p | Status::OPERATED_B).bits().into());
        self.cpu_tick();
    }

    // PulL Accumulator
    fn pla(&mut self, cpu: &mut Cpu) {
        cpu.a = self.pull_stack(cpu);
        cpu.p.set_zn(cpu.a);
        self.cpu_tick();
        self.cpu_tick();
    }

    // PulL Processor status
    fn plp(&mut self, cpu: &mut Cpu) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        cpu.p = Status::from_bits_truncate(self.pull_stack(cpu).into()) & !Status::B | Status::R;
        self.cpu_tick();
        self.cpu_tick();
    }

    // bitwise AND with accumulator
    fn and(&mut self, cpu: &mut Cpu, operand: Operand) {
        let value = self.read(operand);
        cpu.a &= value;
        cpu.p.set_zn(cpu.a);
    }

    // bitwise Exclusive OR
    fn eor(&mut self, cpu: &mut Cpu, operand: Operand) {
        let value = self.read(operand);
        cpu.a ^= value;
        cpu.p.set_zn(cpu.a);
    }

    // bitwise OR with Accumulator
    fn ora(&mut self, cpu: &mut Cpu, operand: Operand) {
        let value = self.read(operand);
        cpu.a |= value;
        cpu.p.set_zn(cpu.a);
    }

    // test BITs
    fn bit(&mut self, cpu: &mut Cpu, operand: Operand) {
        let value = self.read(operand);
        let data = cpu.a & value;
        cpu.p.set(Status::Z, data.u8() == 0);
        cpu.p.set(Status::V, value.nth(6) == 1);
        cpu.p.set(Status::N, value.nth(7) == 1);
    }

    // ADd with Carry
    fn adc(&mut self, cpu: &mut Cpu, operand: Operand) {
        let a = cpu.a;
        let val = self.read(operand);
        let mut result = a + val;

        if cpu.p.contains(Status::C) {
            result += 1;
        }

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let a7 = a.nth(7);
        let v7 = val.nth(7);
        let c6 = a7 ^ v7 ^ result.nth(7);
        let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

        cpu.p.set(Status::C, c7 == 1);
        cpu.p.set(Status::V, (c6 ^ c7) == 1);

        cpu.a = result;
        cpu.p.set_zn(cpu.a)
    }

    // SuBtract with carry
    fn sbc(&mut self, cpu: &mut Cpu, operand: Operand) {
        let a = cpu.a;
        let val = !self.read(operand);
        let mut result = a + val;

        if cpu.p.contains(Status::C) {
            result += 1;
        }

        // http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let a7 = a.nth(7);
        let v7 = val.nth(7);
        let c6 = a7 ^ v7 ^ result.nth(7);
        let c7 = (a7 & v7) | (a7 & c6) | (v7 & c6);

        cpu.p.set(Status::C, c7 == 1);
        cpu.p.set(Status::V, (c6 ^ c7) == 1);

        cpu.a = result;
        cpu.p.set_zn(cpu.a)
    }

    // CoMPare accumulator
    fn cmp(&mut self, cpu: &mut Cpu, operand: Operand) {
        let cmp = Word::from(cpu.a) - Word::from(self.read(operand));
        let cmp_i16 = <Word as Into<i16>>::into(cmp);

        cpu.p.set(Status::C, 0 <= cmp_i16);
        cpu.p.set_zn(cmp_i16 as u16);
    }

    // ComPare X register
    fn cpx(&mut self, cpu: &mut Cpu, operand: Operand) {
        let value = self.read(operand);
        let cmp = cpu.x - value;

        cpu.p.set(Status::C, value <= cpu.x);
        cpu.p.set_zn(cmp);
    }

    // ComPare Y register
    fn cpy(&mut self, cpu: &mut Cpu, operand: Operand) {
        let value = self.read(operand);
        let cmp = cpu.y - value;

        cpu.p.set(Status::C, value <= cpu.y);
        cpu.p.set_zn(cmp);
    }

    // INCrement memory
    fn inc(&mut self, cpu: &mut Cpu, operand: Operand) {
        let result = self.read(operand) + 1;

        cpu.p.set_zn(result);
        self.write(operand, result);
        self.cpu_tick()
    }

    // INcrement X register
    fn inx(&mut self, cpu: &mut Cpu) {
        cpu.x += 1;
        cpu.p.set_zn(cpu.x);
        self.cpu_tick()
    }

    // INcrement Y register
    fn iny(&mut self, cpu: &mut Cpu) {
        cpu.y += 1;
        cpu.p.set_zn(cpu.y);
        self.cpu_tick()
    }

    // DECrement memory
    fn dec(&mut self, cpu: &mut Cpu, operand: Operand) {
        let result = self.read(operand) - 1;

        cpu.p.set_zn(result);
        self.write(operand, result);
        self.cpu_tick()
    }

    // DEcrement X register
    fn dex(&mut self, cpu: &mut Cpu) {
        cpu.x -= 1;
        cpu.p.set_zn(cpu.x);
        self.cpu_tick()
    }

    // DEcrement Y register
    fn dey(&mut self, cpu: &mut Cpu) {
        cpu.y -= 1;
        cpu.p.set_zn(cpu.y);
        self.cpu_tick()
    }

    // Arithmetic Shift Left
    fn asl(&mut self, cpu: &mut Cpu, operand: Operand) {
        let mut data = self.read(operand);

        cpu.p.set(Status::C, data.nth(7) == 1);
        data <<= 1;
        cpu.p.set_zn(data);

        self.write(operand, data);
        self.cpu_tick();
    }

    fn asl_for_accumelator(&mut self, cpu: &mut Cpu) {
        cpu.p.set(Status::C, cpu.a.nth(7) == 1);
        cpu.a <<= 1;
        cpu.p.set_zn(cpu.a);

        self.cpu_tick();
    }

    // Logical Shift Right
    fn lsr(&mut self, cpu: &mut Cpu, operand: Operand) {
        let mut data = self.read(operand);

        cpu.p.set(Status::C, data.nth(0) == 1);
        data >>= 1;
        cpu.p.set_zn(data);

        self.write(operand, data);
        self.cpu_tick();
    }

    fn lsr_for_accumelator(&mut self, cpu: &mut Cpu) {
        cpu.p.set(Status::C, cpu.a.nth(0) == 1);
        cpu.a >>= 1;
        cpu.p.set_zn(cpu.a);

        self.cpu_tick();
    }

    // ROtate Left
    fn rol(&mut self, cpu: &mut Cpu, operand: Operand) {
        let mut data = self.read(operand);
        let c = data.nth(7);

        data <<= 1;
        if cpu.p.contains(Status::C) {
            data |= 0x01;
        }
        cpu.p.set(Status::C, c == 1);
        cpu.p.set_zn(data);
        self.write(operand, data);
        self.cpu_tick();
    }

    fn rol_for_accumelator(&mut self, cpu: &mut Cpu) {
        let c = cpu.a.nth(7);

        let mut a = cpu.a << 1;
        if cpu.p.contains(Status::C) {
            a |= 0x01;
        }
        cpu.a = a;
        cpu.p.set(Status::C, c == 1);
        cpu.p.set_zn(cpu.a);
        self.cpu_tick();
    }

    // ROtate Right
    fn ror(&mut self, cpu: &mut Cpu, operand: Operand) {
        let mut data = self.read(operand);
        let c = data.nth(0);

        data >>= 1;
        if cpu.p.contains(Status::C) {
            data |= 0x80;
        }
        cpu.p.set(Status::C, c == 1);
        cpu.p.set_zn(data);
        self.write(operand, data);
        self.cpu_tick();
    }

    fn ror_for_accumelator(&mut self, cpu: &mut Cpu) {
        let c = cpu.a.nth(0);

        let mut a = cpu.a >> 1;
        if cpu.p.contains(Status::C) {
            a |= 0x80;
        }
        cpu.a = a;
        cpu.p.set(Status::C, c == 1);
        cpu.p.set_zn(cpu.a);
        self.cpu_tick();
    }

    // JuMP
    fn jmp(&mut self, cpu: &mut Cpu, operand: Operand) {
        cpu.pc = operand
    }

    // Jump to SubRoutine
    fn jsr(&mut self, cpu: &mut Cpu, operand: Operand) {
        self.push_stack_word(cpu, cpu.pc - 1);
        self.cpu_tick();
        cpu.pc = operand
    }

    // ReTurn from Subroutine
    fn rts(&mut self, cpu: &mut Cpu) {
        self.cpu_tick();
        self.cpu_tick();
        self.cpu_tick();
        cpu.pc = self.pull_stack_word(cpu) + 1
    }

    // ReTurn from Interrupt
    fn rti(&mut self, cpu: &mut Cpu) {
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.cpu_tick();
        self.cpu_tick();
        cpu.p = Status::from_bits_truncate(self.pull_stack(cpu).into()) & !Status::B | Status::R;
        cpu.pc = self.pull_stack_word(cpu)
    }

    // Branch if Carry Clear
    fn bcc(&mut self, cpu: &mut Cpu, operand: Operand) {
        if !cpu.p.contains(Status::C) {
            self.branch(cpu, operand)
        }
    }

    // Branch if Carry Set
    fn bcs(&mut self, cpu: &mut Cpu, operand: Operand) {
        if cpu.p.contains(Status::C) {
            self.branch(cpu, operand)
        }
    }

    // Branch if EQual
    fn beq(&mut self, cpu: &mut Cpu, operand: Operand) {
        if cpu.p.contains(Status::Z) {
            self.branch(cpu, operand)
        }
    }

    // Branch if MInus
    fn bmi(&mut self, cpu: &mut Cpu, operand: Operand) {
        if cpu.p.contains(Status::N) {
            self.branch(cpu, operand)
        }
    }

    // Branch if NotEqual
    fn bne(&mut self, cpu: &mut Cpu, operand: Operand) {
        if !cpu.p.contains(Status::Z) {
            self.branch(cpu, operand)
        }
    }

    // Branch if PLus
    fn bpl(&mut self, cpu: &mut Cpu, operand: Operand) {
        if !cpu.p.contains(Status::N) {
            self.branch(cpu, operand)
        }
    }

    // Branch if oVerflow Clear
    fn bvc(&mut self, cpu: &mut Cpu, operand: Operand) {
        if !cpu.p.contains(Status::V) {
            self.branch(cpu, operand)
        }
    }

    // Branch if oVerflow Set
    fn bvs(&mut self, cpu: &mut Cpu, operand: Operand) {
        if cpu.p.contains(Status::V) {
            self.branch(cpu, operand)
        }
    }

    // CLear Carry
    fn clc(&mut self, cpu: &mut Cpu) {
        cpu.p.remove(Status::C);
        self.cpu_tick()
    }

    // CLear Decimal
    fn cld(&mut self, cpu: &mut Cpu) {
        cpu.p.remove(Status::D);
        self.cpu_tick()
    }

    // Clear Interrupt
    fn cli(&mut self, cpu: &mut Cpu) {
        cpu.p.remove(Status::I);
        self.cpu_tick()
    }

    // CLear oVerflow
    fn clv(&mut self, cpu: &mut Cpu) {
        cpu.p.remove(Status::V);
        self.cpu_tick()
    }

    // SEt Carry flag
    fn sec(&mut self, cpu: &mut Cpu) {
        cpu.p.insert(Status::C);
        self.cpu_tick()
    }

    // SEt Decimal flag
    fn sed(&mut self, cpu: &mut Cpu) {
        cpu.p |= Status::D;
        self.cpu_tick()
    }

    // SEt Interrupt disable
    fn sei(&mut self, cpu: &mut Cpu) {
        cpu.p.set(Status::I, true);
        self.cpu_tick()
    }

    // BReaK(force interrupt)
    fn brk(&mut self, cpu: &mut Cpu) {
        self.push_stack_word(cpu, cpu.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(cpu, (cpu.p | Status::INTERRUPTED_B).bits().into());
        self.cpu_tick();
        cpu.pc = self.read_word(0xFFFEu16.into());
    }

    // No OPeration
    fn nop(&mut self, _cpu: &mut Cpu) {
        self.cpu_tick();
    }

    fn branch(&mut self, cpu: &mut Cpu, operand: Operand) {
        self.cpu_tick();
        let offset = <Word as Into<u16>>::into(operand) as i8;
        if page_crossed(offset, cpu.pc) {
            self.cpu_tick();
        }
        cpu.pc += offset as u16
    }

    // Load Accumulator and X register
    fn lax(&mut self, cpu: &mut Cpu, operand: Operand) {
        let data = self.read(operand);
        cpu.a = data;
        cpu.x = data;
        cpu.p.set_zn(data);
    }

    // Store Accumulator and X register
    fn sax(&mut self, cpu: &mut Cpu, operand: Operand) {
        self.write(operand, cpu.a & cpu.x)
    }

    // Decrement memory and ComPare to accumulator
    fn dcp(&mut self, cpu: &mut Cpu, operand: Operand) {
        let result = self.read(operand) - 1;
        cpu.p.set_zn(result);
        self.write(operand, result);

        self.cmp(cpu, operand)
    }

    // Increment memory and SuBtract with carry
    fn isb(&mut self, cpu: &mut Cpu, operand: Operand) {
        let result = self.read(operand) + 1;
        cpu.p.set_zn(result);
        self.write(operand, result);

        self.sbc(cpu, operand)
    }

    // arithmetic Shift Left and bitwise Or with accumulator
    fn slo(&mut self, cpu: &mut Cpu, operand: Operand) {
        let mut data = self.read(operand);

        cpu.p.set(Status::C, data.nth(7) == 1);
        data <<= 1;
        cpu.p.set_zn(data);
        self.write(operand, data);

        self.ora(cpu, operand)
    }

    // Rotate Left and bitwise And with accumulator
    fn rla(&mut self, cpu: &mut Cpu, operand: Operand) {
        // rotateLeft excluding tick
        let mut data = self.read(operand);
        let c = data & 0x80;

        data <<= 1;
        if cpu.p.contains(Status::C) {
            data |= 0x01
        }
        cpu.p.remove(Status::C | Status::Z | Status::N);
        cpu.p.set(Status::C, c.u8() == 0x80);
        cpu.p.set_zn(data);

        self.write(operand, data);

        self.and(cpu, operand)
    }

    // logical Shift Right and bitwise Exclusive or
    fn sre(&mut self, cpu: &mut Cpu, operand: Operand) {
        // logicalShiftRight excluding tick
        let mut data = self.read(operand);

        cpu.p.set(Status::C, data.nth(0) == 1);
        data >>= 1;
        cpu.p.set_zn(data);
        self.write(operand, data);

        self.eor(cpu, operand)
    }

    // Rotate Right and Add with carry
    fn rra(&mut self, cpu: &mut Cpu, operand: Operand) {
        // rotateRight excluding tick
        let mut data = self.read(operand);
        let c = data.nth(0);

        data >>= 1;
        if cpu.p.contains(Status::C) {
            data |= 0x80
        }
        cpu.p.set(Status::C, c == 1);
        cpu.p.set_zn(data);

        self.write(operand, data);

        self.adc(cpu, operand)
    }
}

impl Status {
    fn set_zn(&mut self, value: impl Into<u16>) {
        let v: u16 = value.into();
        self.set(Self::Z, v == 0);
        self.set(Self::N, (v >> 7) & 1 == 1);
    }
}
