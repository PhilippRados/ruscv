use crate::cpu::Cpu;
use crate::cpu::Memory;
use crate::get_bits;
use crate::inst_format::*;

use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;

pub enum Inst {
    R(RInst, RFormat),
    I(IInst, IFormat),
    S(SInst, SFormat),
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
    fn op(self) -> Box<dyn FnOnce(u32, u32) -> u32> {
        Box::new(match self {
            RInst::ADD => u32::wrapping_add,
            RInst::SUB => u32::wrapping_sub,
            RInst::XOR => u32::bitxor,
            RInst::OR => u32::bitor,
            RInst::AND => u32::bitand,
            RInst::SLL => |rs1, rs2| {
                let amount = get_bits!(rs2, 0, 4);
                rs1 << amount
            },
            RInst::SRL => |rs1, rs2| {
                let amount = get_bits!(rs2, 0, 4);
                rs1 >> amount
            },
            RInst::SRA => |rs1, rs2| {
                let amount = get_bits!(rs2, 0, 4, i32);
                (rs1 as i32 >> amount as i32) as u32
            },
            RInst::SLT => |rs1, rs2| ((rs1 as i32) < (rs2 as i32)) as u32,
            RInst::SLTU => |rs1, rs2| (rs1 < rs2) as u32,
        })
    }
}
impl From<ArithIInst> for RInst {
    fn from(value: ArithIInst) -> Self {
        match value {
            ArithIInst::ADDI => RInst::ADD,
            ArithIInst::XORI => RInst::XOR,
            ArithIInst::ORI => RInst::OR,
            ArithIInst::ANDI => RInst::AND,
            ArithIInst::SLLI => RInst::SLL,
            ArithIInst::SRLI => RInst::SRL,
            ArithIInst::SRAI => RInst::SRA,
            ArithIInst::SLTI => RInst::SLT,
            ArithIInst::SLTIU => RInst::SLTU,
        }
    }
}

pub enum ArithIInst {
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

pub enum MemIInst {
    LB,
    LH,
    LW,
    LBU,
    LHU,
}
impl MemIInst {
    fn op(self, mem: &Memory) -> Box<dyn FnOnce(u32, u32) -> u32 + '_> {
        let size_bytes = match &self {
            MemIInst::LB | MemIInst::LBU => 1,
            MemIInst::LH | MemIInst::LHU => 2,
            MemIInst::LW => 4,
        };
        let mem = &mem.0;
        let zero_extends = matches!(self, MemIInst::LBU | MemIInst::LHU);
        Box::new(move |rs1, imm| {
            let from = u32::wrapping_add(rs1, imm);
            let to = u32::wrapping_add(from, size_bytes);
            let value = mem[from as usize..to as usize].try_into().unwrap();
            if zero_extends {
                u32::from_le_bytes(value)
            } else {
                i32::from_le_bytes(value) as u32
            }
        })
    }
}

pub enum IInst {
    Arith(ArithIInst),
    Mem(MemIInst),
}

pub enum SInst {
    SB,
    SH,
    SW,
}

impl SInst {
    fn op(self, mem: &mut Memory) -> Box<dyn FnOnce(u32, u32, u32) + '_> {
        let size_bits: usize = match &self {
            SInst::SB => 8,
            SInst::SH => 16,
            SInst::SW => 32,
        };

        Box::new(move |rs1, rs2, imm| {
            let base = u32::wrapping_add(rs1, imm);
            for i in (0..size_bits).step_by(8) {
                let address = u32::wrapping_add(base, i as u32);
                mem.0[address as usize] = get_bits!(rs2, i, i + 7) as u8;
            }
        })
    }
}

impl Inst {
    pub fn execute(self, cpu: &mut Cpu) {
        // TODO: Do these actually have to be closures? Why not compute values directly
        match self {
            Inst::R(inst, format) => {
                let alu = inst.op();
                let result = alu(cpu.read_reg(format.rs1), cpu.read_reg(format.rs2));
                cpu.write_reg(format.rd, result);
            }
            Inst::I(inst, format) => {
                let alu = match inst {
                    // Arithmetic operations are the same for R/I format, only the second operand differs.
                    IInst::Arith(inst) => RInst::from(inst).op(),
                    IInst::Mem(inst) => inst.op(&cpu.mem),
                };

                let result = alu(cpu.read_reg(format.rs1), format.imm);
                cpu.write_reg(format.rd, result);
            }
            Inst::S(inst, format) => {
                let rs1 = cpu.read_reg(format.rs1);
                let rs2 = cpu.read_reg(format.rs2);
                let alu = inst.op(&mut cpu.mem);
                alu(rs1, rs2, format.imm);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_assigns_byte() {
        let mut cpu = Cpu::new();
        cpu.write_reg(28, 12);
        // mem[0 + 3] = 12[0:7]
        let inst = Inst::S(
            SInst::SB,
            SFormat {
                funct3: 0x0,
                rs1: 0,
                rs2: 28,
                imm: 3,
            },
        );
        inst.execute(&mut cpu);
        assert_eq!(cpu.mem.0[3], 12);
    }
}
