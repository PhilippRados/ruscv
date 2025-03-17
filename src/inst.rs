use crate::cpu::Cpu;
use crate::inst_format::*;

use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;

pub enum Inst {
    R(RInst, RFormat),
    I(IInst, IFormat),
}

pub enum RInst {
    ADD,
    SUB,
    XOR,
    OR,
    AND,
    SLL,
    SRL,
    SRA,
    SLT,
    SLTU,
}
impl RInst {
    fn op(self) -> fn(u32, u32) -> u32 {
        match self {
            RInst::ADD => u32::wrapping_add,
            RInst::SUB => u32::wrapping_sub,
            RInst::XOR => u32::bitxor,
            RInst::OR => u32::bitor,
            RInst::AND => u32::bitand,
            RInst::SLL => |rs1, rs2| {
                let amount = get_bits(rs2, 0, 4);
                rs1 << amount
            },
            RInst::SRL => |rs1, rs2| {
                let amount = get_bits(rs2, 0, 4);
                rs1 >> amount
            },
            RInst::SRA => |rs1, rs2| {
                let amount = get_bits(rs2, 0, 4);
                (rs1 as i32 >> amount as i32) as u32
            },
            RInst::SLT => |rs1, rs2| ((rs1 as i32) < (rs2 as i32)) as u32,
            RInst::SLTU => |rs1, rs2| (rs1 < rs2) as u32,
        }
    }
}
impl From<IInst> for RInst {
    fn from(value: IInst) -> Self {
        match value {
            IInst::ADDI => RInst::ADD,
            IInst::XORI => RInst::XOR,
            IInst::ORI => RInst::OR,
            IInst::ANDI => RInst::AND,
            IInst::SLLI => RInst::SLL,
            IInst::SRLI => RInst::SRL,
            IInst::SRAI => RInst::SRA,
            IInst::SLTI => RInst::SLT,
            IInst::SLTIU => RInst::SLTU,
        }
    }
}

pub enum IInst {
    ADDI,
    XORI,
    ORI,
    ANDI,
    SLLI,
    SRLI,
    SRAI,
    SLTI,
    SLTIU,
}

impl Inst {
    pub fn execute(self, cpu: &mut Cpu) {
        match self {
            Inst::R(inst, format) => {
                let alu = inst.op();
                let result = alu(cpu.read_reg(format.rs1), cpu.read_reg(format.rs2));
                cpu.write_reg(format.rd, result);
            }
            Inst::I(inst, format) => {
                // operations are the same for R/I format, only the second operand differs
                let alu = RInst::from(inst).op();
                let result = alu(cpu.read_reg(format.rs1), format.imm12);
                cpu.write_reg(format.rd, result);
            }
        }
    }
}
