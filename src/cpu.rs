use crate::error::Error;
use crate::get_bits;
use crate::inst::*;
use crate::inst_format::*;

const MEMSIZE: usize = 1024;
const INSTSIZE_BYTES: usize = 4;

pub struct Cpu {
    regs: [u32; 32],
    pc: u32,
    mem: [u8; MEMSIZE], // should wrap around
}

impl Cpu {
    pub fn new() -> Self {
        let mut regs = [0; 32];
        // initializes stack pointer to top of stack
        regs[2] = MEMSIZE as u32;

        Cpu {
            regs,
            pc: 0,
            mem: [0; MEMSIZE],
        }
    }

    pub fn run(mut self, program: Vec<u8>) -> Result<(), Error> {
        let program_cycles = program.len() / INSTSIZE_BYTES;
        self.load_program(program);

        for cycle in 0..program_cycles {
            self.emulate_cycle()?;
            self.dump_state(cycle);
        }

        Ok(())
    }

    pub fn read_reg(&self, reg_idx: usize) -> u32 {
        assert!(reg_idx < 32, "rv32i only has 32 registers");
        if reg_idx == 0 {
            return 0;
        }

        self.regs[reg_idx]
    }
    pub fn write_reg(&mut self, reg_idx: usize, value: u32) {
        assert!(reg_idx < 32, "rv32i only has 32 registers");
        if reg_idx == 0 {
            return;
        }

        self.regs[reg_idx] = value;
    }

    fn dump_state(&self, cycle_count: usize) {
        eprintln!("CPU dump at cycle {cycle_count}:");
        eprintln!("PC: {}", self.pc);
        for i in 0..32 {
            eprintln!("R{i}: {}", self.regs[i] as i32);
        }
    }

    // loads program to start of the memory
    fn load_program(&mut self, mut program: Vec<u8>) {
        program.resize_with(MEMSIZE, || 0);
        self.mem = program.as_slice().try_into().unwrap()
    }

    // fetches a 32 bit instruction from memory
    fn fetch(&mut self) -> Result<u32, Error> {
        let pc = self.pc as usize;
        self.pc += INSTSIZE_BYTES as u32;
        if pc > MEMSIZE - INSTSIZE_BYTES {
            return Err(Error::InvalidPC);
        }

        // return instruction in little-endian
        Ok(u32::from_le_bytes(self.mem[pc..pc + 4].try_into().unwrap()))
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
                    _ => return Err(Error::InvalidInstFormat(Box::new(r_format))),
                };

                Inst::R(inst, r_format)
            }
            0b0010011 => {
                let i_format = IFormat::new(raw_inst);
                let upper_imm = get_bits!(i_format.imm12, 5, 11);
                let inst = match (i_format.funct3, upper_imm) {
                    (0x0, _) => IInst::ADDI,
                    (0x4, _) => IInst::XORI,
                    (0x6, _) => IInst::ORI,
                    (0x7, _) => IInst::ANDI,
                    (0x1, 0x00) => IInst::SLLI,
                    (0x5, 0x00) => IInst::SRLI,
                    (0x5, 0x20) => IInst::SRAI,
                    (0x2, _) => IInst::SLTI,
                    (0x3, _) => IInst::SLTIU,
                    _ => return Err(Error::InvalidInstFormat(Box::new(i_format))),
                };

                Inst::I(inst, i_format)
            }
            _ => return Err(Error::InvalidOpcode(opcode)),
        };

        Ok(inst)
    }

    fn emulate_cycle(&mut self) -> Result<(), Error> {
        let raw_inst = self.fetch()?;
        let inst = self.decode(raw_inst)?;
        inst.execute(self);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::process::Command;

    fn asm_to_bin(asm: &'static str) -> Vec<u8> {
        let mut asm_temp = tempfile::Builder::new()
            .suffix(".s")
            .tempfile()
            .expect("tempfile create");
        write!(asm_temp, "{}", asm).expect("write asm to tempfile");

        let executable = tempfile::NamedTempFile::new().expect("tempfile create");
        Command::new("riscv64-unknown-elf-gcc")
            .args([
                "-Wl,-Ttext=0x0",
                "-nostdlib",
                "-o",
                executable.path().to_str().unwrap(),
                asm_temp.path().to_str().unwrap(),
                "-march=rv32i",
                "-mabi=ilp32",
            ])
            .output()
            .expect("invokes riscv gcc cross compiler");

        let binary = tempfile::NamedTempFile::new().expect("tempfile create");
        Command::new("riscv64-unknown-elf-objcopy")
            .args([
                "-O",
                "binary",
                executable.path().to_str().unwrap(),
                binary.path().to_str().unwrap(),
            ])
            .output()
            .expect("invokes riscv objcopy");

        crate::read_bin(binary.path().to_str().unwrap())
    }

    #[test]
    fn x0_hardwired() {
        let program = asm_to_bin("addi x0, x0, -127\n");
        let mut cpu = Cpu::new();
        cpu.load_program(program);

        assert!(cpu.emulate_cycle().is_ok());
        assert_eq!(0, cpu.read_reg(0));
    }

    #[test]
    fn negative_assign() {
        let program = asm_to_bin("addi x31, x0, -127\n");
        let mut cpu = Cpu::new();
        cpu.load_program(program);

        assert!(cpu.emulate_cycle().is_ok());
        let n = -127;
        assert_eq!(n as u32, cpu.read_reg(31));
        assert_eq!(0, cpu.read_reg(0));
    }
}
