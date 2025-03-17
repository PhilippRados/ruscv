use crate::cpu::Cpu;
use crate::inst_format::*;

pub enum Inst {
    R(RInst, RFormat),
    I(IInst, IFormat),
}

pub enum RInst {
    ADD,
}
impl RInst {
    fn func(self) -> fn(u32, u32) -> u32 {
        match self {
            RInst::ADD => u32::wrapping_add,
        }
    }
}

pub enum IInst {
    ADDI,
}

impl IInst {
    fn func(self) -> fn(u32, u32) -> u32 {
        match self {
            IInst::ADDI => u32::wrapping_add,
        }
    }
}

impl Inst {
    pub fn execute(self, cpu: &mut Cpu) {
        match self {
            Inst::R(inst, format) => {
                let alu = inst.func();
                let result = alu(cpu.read_reg(format.rs1), cpu.read_reg(format.rs2));
                cpu.write_reg(format.rd, result);
            }
            Inst::I(inst, format) => {
                let alu = inst.func();
                let result = alu(cpu.read_reg(format.rs1), format.imm12);
                cpu.write_reg(format.rd, result);
            }
        }
    }
}
