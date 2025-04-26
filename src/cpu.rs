use crate::error::*;
use crate::get_bits;
use crate::inst::*;
use crate::inst_format::*;
use crate::memory::*;
use crate::pc::*;
use crate::regs::*;

// Don't want to use too much memory for emulator
pub const MEMSIZE: usize = 1024 * 128;
// Start address of dram section
const MEM_START: u32 = 0x8000_0000;

pub const INSTSIZE_BYTES: usize = 4;

enum ProgState {
    Continue,
    Exit(u8),
}

pub struct Cpu {
    pub pc: ProgramCounter,
    pub regs: Registers,
    pub mem: Memory,
    print_debug: bool,
}

impl Cpu {
    pub fn new(print_debug: bool) -> Self {
        Cpu {
            print_debug,
            pc: ProgramCounter::new(),
            regs: Registers::new(),
            mem: Memory::new(),
        }
    }

    pub fn run(&mut self, program: Vec<u8>) -> Result<u8, Error> {
        self.mem.load_program(program);

        for cycle in 0.. {
            match self.emulate_cycle() {
                Ok(ProgState::Exit(code)) => {
                    self.dump_state(cycle);
                    return Ok(code);
                }
                Err(e) => {
                    self.dump_state(cycle);
                    return Err(e);
                }
                _ => (),
            }
            if self.print_debug {
                self.dump_state(cycle);
            }
        }

        unreachable!("Emulator should either run out of instructions or exit using syscall")
    }

    fn dump_state(&self, cycle_count: usize) {
        eprintln!("CPU dump at cycle {cycle_count}:");
        eprintln!("PC: {}", self.pc.get());
        for i in 0..32 {
            eprintln!("R{i}: {}", self.regs.read(i) as i32);
        }
    }

    // fetches next instruction from memory
    fn fetch(&mut self) -> Result<u32, Error> {
        let pc = self.pc.inc()?;
        Ok(self.mem.read(Size::Word, pc, true))
    }

    // parses raw byte instruction into correct format
    // for decode information see: [riscv-ref](crate::docs/riscv-ref)
    fn decode(&self, raw_inst: u32) -> Result<Inst, Error> {
        // get the lowest 7 bits for the opcode
        let opcode = get_bits!(raw_inst, 0, 6);
        let inst = match opcode {
            0b0110011 => {
                let r_format = RFormat::new(raw_inst);
                let inst = match (r_format.funct3, r_format.funct7) {
                    (0x0, 0x00) => RInst::ADD,
                    (0x0, 0x20) => RInst::SUB,
                    (0x4, 0x00) => RInst::XOR,
                    (0x6, 0x00) => RInst::OR,
                    (0x7, 0x00) => RInst::AND,
                    (0x1, 0x00) => RInst::SLL,
                    (0x5, 0x00) => RInst::SRL,
                    (0x5, 0x20) => RInst::SRA,
                    (0x2, 0x00) => RInst::SLT,
                    (0x3, 0x00) => RInst::SLTU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::R(r_format))),
                };

                Inst::R(inst, r_format)
            }
            0b0010011 => {
                let i_format = IFormat::new(raw_inst);
                let upper_imm = get_bits!(i_format.imm, 5, 11);
                let inst = match (i_format.funct3, upper_imm) {
                    (0x0, _) => ArithIInst::ADDI,
                    (0x4, _) => ArithIInst::XORI,
                    (0x6, _) => ArithIInst::ORI,
                    (0x7, _) => ArithIInst::ANDI,
                    (0x1, 0x00) => ArithIInst::SLLI,
                    (0x5, 0x00) => ArithIInst::SRLI,
                    (0x5, 0x20) => ArithIInst::SRAI,
                    (0x2, _) => ArithIInst::SLTI,
                    (0x3, _) => ArithIInst::SLTIU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::I(i_format))),
                };

                Inst::I(IInst::Arith(inst), i_format)
            }
            0b0000011 => {
                let i_format = IFormat::new(raw_inst);
                let inst = match i_format.funct3 {
                    0x0 => LoadIInst::LB,
                    0x1 => LoadIInst::LH,
                    0x2 => LoadIInst::LW,
                    0x4 => LoadIInst::LBU,
                    0x5 => LoadIInst::LHU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::I(i_format))),
                };

                Inst::I(IInst::Mem(inst), i_format)
            }
            0b1100111 => {
                let i_format = IFormat::new(raw_inst);
                if let 0x0 = i_format.funct3 {
                    Inst::I(IInst::Jalr, i_format)
                } else {
                    return Err(Error::InvalidInstFormat(FormatError::I(i_format)));
                }
            }
            0b0100011 => {
                let s_format = SFormat::new(raw_inst);
                let inst = match s_format.funct3 {
                    0x0 => SInst::SB,
                    0x1 => SInst::SH,
                    0x2 => SInst::SW,
                    _ => return Err(Error::InvalidInstFormat(FormatError::S(s_format))),
                };

                Inst::S(inst, s_format)
            }
            0b1100011 => {
                let b_format = BFormat::new(raw_inst);
                let inst = match b_format.funct3 {
                    0x0 => BInst::BEQ,
                    0x1 => BInst::BNE,
                    0x4 => BInst::BLT,
                    0x5 => BInst::BGE,
                    0x6 => BInst::BLTU,
                    0x7 => BInst::BGEU,
                    _ => return Err(Error::InvalidInstFormat(FormatError::B(b_format))),
                };

                Inst::B(inst, b_format)
            }
            0b1101111 => {
                // jal instruction is the only J-Format instruction
                Inst::J(JFormat::new(raw_inst))
            }
            0b0110111 => Inst::U(UInst::LUI, UFormat::new(raw_inst)),
            0b0010111 => Inst::U(UInst::AUIPC, UFormat::new(raw_inst)),
            0b1110011 => {
                // ecall
                let call = if self.regs.read(17) == 93 {
                    // intercept exit syscall (a7 == 93) to check official risc-v testsuite
                    SysCall::Exit(self.regs.read(10) as u8)
                } else {
                    SysCall::Nop
                };
                Inst::SysCall(call)
            }
            0b0001111 => {
                // fence (also necessary for riscv-tests)
                Inst::SysCall(SysCall::Nop)
            }
            _ => return Err(Error::InvalidOpcode(opcode)),
        };

        Ok(inst)
    }

    fn emulate_cycle(&mut self) -> Result<ProgState, Error> {
        let raw_inst = self.fetch()?;
        if raw_inst == 0 {
            return Err(Error::EndOfInstructions);
        }
        if self.print_debug {
            eprintln!("Inst: {:032b}", raw_inst);
        }

        let inst = self.decode(raw_inst)?;
        if let Inst::SysCall(SysCall::Exit(code)) = inst {
            return Ok(ProgState::Exit(code));
        }

        inst.execute(self);
        Ok(ProgState::Continue)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::path::Path;
    use std::process::Command;

    // NOTE: The testcases in tests/ terminate by running out of instructions.
    // This is by design, as I don't want to exit each testcase using ecall.

    fn file_to_bin(path: &'static str) -> Vec<u8> {
        let mut current_path = std::env::current_dir().unwrap();
        current_path.push("tests");
        current_path.push(path);
        create_bin(current_path.as_path())
    }
    fn asm_to_bin(asm: &'static str) -> Vec<u8> {
        let mut asm_temp = tempfile::Builder::new()
            .suffix(".s")
            .tempfile()
            .expect("tempfile create");
        write!(asm_temp, ".global _start\n_start:\n{}", asm).expect("write asm to tempfile");
        create_bin(asm_temp.path())
    }

    fn create_bin(asm_filepath: &Path) -> Vec<u8> {
        let executable = tempfile::NamedTempFile::new().expect("tempfile create");
        assert!(
            Command::new("riscv64-unknown-elf-gcc")
                .args([
                    "-Wl,-Ttext=0x0",
                    "-nostdlib",
                    "-o",
                    executable.path().to_str().unwrap(),
                    asm_filepath.to_str().unwrap(),
                    "-march=rv32i",
                    "-mabi=ilp32",
                ])
                .status()
                .expect("invokes riscv gcc cross compiler")
                .success(),
            "invalid asm"
        );

        let binary = tempfile::NamedTempFile::new().expect("tempfile create");
        assert!(
            Command::new("riscv64-unknown-elf-objcopy")
                .args([
                    "-O",
                    "binary",
                    executable.path().to_str().unwrap(),
                    binary.path().to_str().unwrap(),
                ])
                .status()
                .expect("invokes riscv objcopy")
                .success(),
            "invalid elf"
        );

        crate::read_bin(binary.path().to_str().unwrap())
    }

    #[test]
    fn x0_hardwired() {
        let program = asm_to_bin("addi x0, x0, -127\n");
        let mut cpu = Cpu::new(false);
        cpu.mem.load_program(program);

        assert!(cpu.emulate_cycle().is_ok());
        assert_eq!(0, cpu.regs.read(0));
    }

    #[test]
    fn negative_assign() {
        let program = asm_to_bin("addi x31, x0, -127\n");
        let mut cpu = Cpu::new(false);
        cpu.mem.load_program(program);

        assert!(cpu.emulate_cycle().is_ok());
        let n = -127;
        assert_eq!(n as u32, cpu.regs.read(31));
        assert_eq!(0, cpu.regs.read(0));
    }

    #[test]
    fn auipc_copy() {
        let program = asm_to_bin("auipc x10, 0\n");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(10), 0);
    }

    #[test]
    fn auipc_offset() {
        let program = asm_to_bin("addi x11, x0, 12\nauipc x10, 4\n");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(10), 16388);
    }

    #[test]
    fn arithmetic() {
        let program = file_to_bin("arith.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(27) as i32, -26);
        assert_eq!(cpu.regs.read(28) as i32, -6);
        assert_eq!(cpu.regs.read(29), 5);
        assert_eq!(cpu.regs.read(30) as i32, -32);
        assert_eq!(cpu.regs.read(31) as i32, 42);
        assert_eq!(cpu.pc.get(), 28);
    }

    #[test]
    fn bitops() {
        let program = file_to_bin("bitops.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(28) as i32, 1);
        assert_eq!(cpu.regs.read(29), 5);
        assert_eq!(cpu.regs.read(30) as i32, -123);
        assert_eq!(cpu.regs.read(31), 0);
        assert_eq!(cpu.pc.get(), 24);
    }
    #[test]
    fn load() {
        let program = file_to_bin("load.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(27), 60);
        assert_eq!(cpu.regs.read(30), 60);
        assert_eq!(cpu.regs.read(29), 60);
        assert_eq!(cpu.regs.read(28), 60);
        assert_eq!(cpu.mem.read(Size::Byte, 64, true), 60);
    }

    #[test]
    fn negative_load_imm() {
        let program = file_to_bin("negative_load.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(27), 21);
        assert_eq!(cpu.regs.read(28), 60);
        assert_eq!(cpu.regs.read(30), 60);
        assert_eq!(cpu.mem.read(Size::Byte, 20, true), 60);
    }

    #[test]
    fn negative_store_imm() {
        let program = file_to_bin("negative_store.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(22), 261);
        assert_eq!(cpu.regs.read(27), 256);
        assert_eq!(cpu.regs.read(28), 60);
        assert_eq!(cpu.regs.read(30), 60);
        assert_eq!(cpu.mem.read(Size::Byte, 256, true), 60);
    }

    #[test]
    fn branch_eq() {
        let program = file_to_bin("branch.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(20) as i32, -2);
        assert_eq!(cpu.regs.read(21), 1);
    }

    #[test]
    fn branch_signed() {
        let program = file_to_bin("signed_branch.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(20) as i32, -1);
        assert_eq!(cpu.regs.read(21), 1);
    }
    #[test]
    fn branch_unsigned() {
        let program = file_to_bin("unsigned_branch.s");
        let mut cpu = Cpu::new(false);

        assert!(matches!(cpu.run(program), Err(Error::EndOfInstructions)));
        assert_eq!(cpu.regs.read(20), 100);
        assert_eq!(cpu.regs.read(21), 100);
    }

    #[test]
    fn fibonacci() {
        let program = file_to_bin("fibs.s");
        let mut cpu = Cpu::new(false);

        // fibonacci terminates using exit syscall which is why result is Ok.
        assert!(cpu.run(program).is_ok());
        //  fibs(10) == a0 == r10 == 55
        assert_eq!(cpu.regs.read(10), 55);
    }
}
