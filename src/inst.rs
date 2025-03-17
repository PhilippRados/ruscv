use crate::cpu::Cpu;
use crate::inst_format::*;

pub enum Inst {
    ADD(RFormat),
    ADDI(IFormat),
}

impl Inst {
    pub fn execute(self, cpu: &mut Cpu) {
        match self {
            Inst::ADD(format) => cpu.write_reg(
                format.rd,
                cpu.read_reg(format.rs1)
                    .wrapping_add(cpu.read_reg(format.rs2)),
            ),
            Inst::ADDI(format) => cpu.write_reg(
                format.rd,
                cpu.read_reg(format.rs1).wrapping_add(format.imm12),
            ),
        }
    }
}
