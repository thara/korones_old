use super::*;

use crate::bus::*;

trait IntrruptHandler: CpuTick + Bus + CpuStack {
    fn reset(&mut self, cpu: &mut Cpu) {
        cpu.cycles += 5;
        cpu.pc = self.read_word(0xFFFCu16.into());
        cpu.p.insert(Status::I);
        cpu.s -= 3;
    }

    // NMI
    fn non_markable_interrupt(&mut self, cpu: &mut Cpu) {
        cpu.cycles += 2;
        self.push_stack_word(cpu, cpu.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(cpu, (cpu.p | Status::INTERRUPTED_B).bits().into());
        cpu.p.insert(Status::I);
        cpu.pc = self.read_word(0xFFFAu16.into())
    }

    // IRQ
    fn interrupt_request(&mut self, cpu: &mut Cpu) {
        cpu.cycles += 2;
        self.push_stack_word(cpu, cpu.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(cpu, (cpu.p | Status::INTERRUPTED_B).bits().into());
        cpu.p.insert(Status::I);
        cpu.pc = self.read_word(0xFFFEu16.into())
    }

    // BRK
    fn break_interrupt(&mut self, cpu: &mut Cpu) {
        cpu.cycles += 2;
        cpu.pc += 1;
        self.push_stack_word(cpu, cpu.pc);
        // https://wiki.nesdev.com/w/index.php/Status_flags#The_B_flag
        // http://visual6502.org/wiki/index.php?title=6502_BRK_and_B_bit
        self.push_stack(cpu, (cpu.p | Status::INTERRUPTED_B).bits().into());
        cpu.p.insert(Status::I);
        cpu.pc = self.read_word(0xFFFEu16.into())
    }
}
