use crate::error::Error;
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
            eprintln!("R{i}: {}", self.regs[i]);
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
        let opcode = get_bits(raw_inst, 0, 6);
        let inst = match opcode {
            0b0110011 => {
                let r_format = RFormat::new(raw_inst);
                let inst = match (r_format.funct3, r_format.funct7) {
                    (0x0, 0x00) => RInst::ADD,
                    _ => return Err(Error::InvalidInstFormat(Box::new(r_format))),
                };

                Inst::R(inst, r_format)
            }
            0b0010011 => {
                let i_format = IFormat::new(raw_inst);
                let inst = match i_format.funct3 {
                    0x0 => IInst::ADDI,
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
