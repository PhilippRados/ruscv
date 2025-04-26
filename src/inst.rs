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
    U(UInst, UFormat),

    // This isn't an official instruction but just so that the emulator doesn't crash on `ecall`.
    // Only handles exit for now, every other syscall is ignored.
    SysCall(SysCall),
}

pub enum SysCall {
    Exit(u8),
    Nop,
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
    fn op(self) -> impl FnOnce(u32, u32) -> u32 {
        match self {
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
        }
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
macro_rules! load_mem {
    ($ty:ty,$mem:expr,$from:expr,$to:expr) => {
        <$ty>::from_le_bytes($mem[$from as usize..$to as usize].try_into().unwrap()) as u32
    };
}
impl MemIInst {
    fn op(self, mem: &Memory) -> impl FnOnce(u32, u32) -> u32 + '_ {
        let size_bytes = match &self {
            MemIInst::LB | MemIInst::LBU => 1,
            MemIInst::LH | MemIInst::LHU => 2,
            MemIInst::LW => 4,
        };
        let mem = &mem.0;
        move |rs1, imm| {
            let from = u32::wrapping_add(rs1, imm);
            let to = u32::wrapping_add(from, size_bytes);
            match self {
                MemIInst::LBU => load_mem!(u8, mem, from, to),
                MemIInst::LHU => load_mem!(u16, mem, from, to),
                MemIInst::LW => load_mem!(u32, mem, from, to),
                MemIInst::LB => load_mem!(i8, mem, from, to),
                MemIInst::LH => load_mem!(i16, mem, from, to),
            }
        }
    }
}

pub enum IInst {
    Arith(ArithIInst),
    Mem(MemIInst),
    Jalr,
}
impl IInst {
    fn op(self, cpu: &mut Cpu) -> Box<dyn FnOnce(u32, u32) -> u32 + '_> {
        match self {
            // Arithmetic operations are the same for R/I format, only the second operand differs.
            IInst::Arith(inst) => Box::new(RInst::from(inst).op()),
            IInst::Mem(inst) => Box::new(inst.op(&cpu.mem)),
            IInst::Jalr => Box::new(|rs1, imm| {
                let original_pc = cpu.pc;
                cpu.pc = u32::wrapping_add(rs1, imm);
                original_pc
            }),
        }
    }
}

pub enum SInst {
    SB,
    SH,
    SW,
}

impl SInst {
    fn op(self, mem: &mut Memory) -> impl FnOnce(u32, u32, u32) + '_ {
        let size_bytes: usize = match &self {
            SInst::SB => 1,
            SInst::SH => 2,
            SInst::SW => 4,
        };

        move |rs1, rs2, imm| {
            let base = u32::wrapping_add(rs1, imm);
            for i in 0..size_bytes {
                let address = u32::wrapping_add(base, i as u32);
                let bit_offset = i * 8;
                mem.0[address as usize] = get_bits!(rs2, bit_offset, bit_offset + 7) as u8;
            }
        }
    }
}

pub enum BInst {
    BEQ,
    BNE,
    BLT,
    BGE,
    BLTU,
    BGEU,
}

pub enum UInst {
    LUI,
    AUIPC,
}
impl UInst {
    fn op(self, pc: u32) -> impl FnOnce(u32) -> u32 {
        move |imm| match self {
            UInst::LUI => imm << 12,
            UInst::AUIPC => u32::wrapping_add(pc - 4, imm << 12),
        }
    }
}

impl Inst {
    pub fn execute(self, cpu: &mut Cpu) {
        match self {
            Inst::R(inst, format) => {
                let alu = inst.op();
                let result = alu(cpu.regs.read(format.rs1), cpu.regs.read(format.rs2));
                cpu.regs.write(format.rd, result);
            }
            Inst::I(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let alu = inst.op(cpu);
                let result = alu(rs1, format.imm);
                cpu.regs.write(format.rd, result);
            }
            Inst::S(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let rs2 = cpu.regs.read(format.rs2);
                let alu = inst.op(&mut cpu.mem);
                alu(rs1, rs2, format.imm);
            }
            Inst::B(inst, format) => {
                let rs1 = cpu.regs.read(format.rs1);
                let rs2 = cpu.regs.read(format.rs2);
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
                cpu.regs.write(format.rd, cpu.pc);
                cpu.pc =
                    u32::wrapping_add(cpu.pc, u32::wrapping_sub(format.imm, INSTSIZE_BYTES as u32));
            }
            Inst::U(inst, format) => {
                let alu = inst.op(cpu.pc);
                let result = alu(format.imm);
                cpu.regs.write(format.rd, result);
            }
            Inst::SysCall(..) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn store_assigns_byte() {
        let mut cpu = Cpu::new(false);
        cpu.regs.write(28, 12);
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

    #[test]
    fn lui() {
        let mut cpu = Cpu::new(false);

        let inst = Inst::U(UInst::LUI, UFormat { rd: 10, imm: 1 });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 4096);

        let inst = Inst::U(UInst::LUI, UFormat { rd: 10, imm: 3 });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 12288);

        let inst = Inst::U(UInst::LUI, UFormat { rd: 10, imm: 0x100 });
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 1048576);
    }

    #[test]
    fn lui_max() {
        let mut cpu = Cpu::new(false);
        let inst = Inst::U(
            UInst::LUI,
            UFormat {
                rd: 10,
                imm: 0b1111_1111_1111_1111,
            },
        );
        inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 0b1111_1111_1111_1111_0000_0000_0000);
    }

    #[test]
    fn long_jump() {
        // manually test really big addresses, since emulator only has little memory.
        // auipc x5, 0x03000
        // jalr x10, x5, -0x400

        let mut cpu = Cpu::new(false);
        // pc was already incremented by fetch so emulate that.
        cpu.pc = 0x40000004;
        let auipc_inst = Inst::U(
            UInst::AUIPC,
            UFormat {
                rd: 5,
                imm: 0x03000,
            },
        );
        auipc_inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(5), 0x43000000);

        // manually increment pc since no fetch phase
        cpu.pc += 4;

        let jalr_inst = Inst::I(
            IInst::Jalr,
            IFormat {
                rd: 10,
                funct3: 0,
                rs1: 5,
                imm: -0x400i32 as u32,
            },
        );
        jalr_inst.execute(&mut cpu);
        assert_eq!(cpu.regs.read(10), 0x40000008);
        assert_eq!(cpu.pc, 0x42fffc00);
    }
}
