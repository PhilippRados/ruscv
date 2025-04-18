use crate::cpu::*;
use crate::get_bits;
use crate::inst_format::*;

use std::ops::BitAnd;
use std::ops::BitOr;
use std::ops::BitXor;

pub enum Inst {
    R(RInst, RFormat),
    I(IInst, IFormat),
    S(SInst, SFormat),
    B(BInst, BFormat),
    J(JFormat),
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
    Jump,
}

pub enum SInst {
    SB,
    SH,
    SW,
}

impl SInst {
    fn op(self, mem: &mut Memory) -> Box<dyn FnOnce(u32, u32, u32) + '_> {
        let size_bytes: usize = match &self {
            SInst::SB => 1,
            SInst::SH => 2,
            SInst::SW => 4,
        };

        Box::new(move |rs1, rs2, imm| {
            let base = u32::wrapping_add(rs1, imm);
            for i in 0..size_bytes {
                let address = u32::wrapping_add(base, i as u32);
                let bit_offset = i * 8;
                mem.0[address as usize] = get_bits!(rs2, bit_offset, bit_offset + 7) as u8;
            }
        })
    }
}

#[derive(Debug)]
pub enum BInst {
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
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
                let mut is_jump = false;
                let alu = match inst {
                    // Arithmetic operations are the same for R/I format, only the second operand differs.
                    IInst::Arith(inst) => RInst::from(inst).op(),
                    IInst::Mem(inst) => inst.op(&cpu.mem),
                    IInst::Jump => {
                        is_jump = true;
                        Box::new(|_, _| cpu.pc)
                    }
                };

                let result = alu(cpu.read_reg(format.rs1), format.imm);
                cpu.write_reg(format.rd, result);

                if is_jump {
                    cpu.pc = u32::wrapping_add(cpu.read_reg(format.rs1), format.imm);
                }
            }
            Inst::S(inst, format) => {
                let rs1 = cpu.read_reg(format.rs1);
                let rs2 = cpu.read_reg(format.rs2);
                let alu = inst.op(&mut cpu.mem);
                alu(rs1, rs2, format.imm);
            }
            Inst::B(inst, format) => {
                let rs1 = cpu.read_reg(format.rs1);
                let rs2 = cpu.read_reg(format.rs2);
                let branch = match inst {
                    BInst::BEQ => rs1 == rs2,
                    BInst::BNE => rs1 != rs2,
                    BInst::BLT => rs1 as i32 <= rs2 as i32,
                    BInst::BLTU => rs1 <= rs2,
                    BInst::BGE => rs1 as i32 >= rs2 as i32,
                    BInst::BGEU => rs1 >= rs2,
                };
                if branch {
                    cpu.pc = u32::wrapping_add(
                        cpu.pc,
                        u32::wrapping_sub(format.imm, INSTSIZE_BYTES as u32),
                    );
                }
            }
            Inst::J(format) => {
                cpu.write_reg(format.rd, cpu.pc);
                cpu.pc =
                    u32::wrapping_add(cpu.pc, u32::wrapping_sub(format.imm, INSTSIZE_BYTES as u32));
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
