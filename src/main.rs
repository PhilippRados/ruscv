use std::fmt;
use std::fs::File;
use std::io::Read;

const MEMSIZE: usize = 1024;
const INSTSIZE_BYTES: usize = 4;

struct RFormat {
    rd: usize,
    funct3: usize,
    rs1: usize,
    rs2: usize,
    funct7: usize,
}
impl RFormat {
    fn new(raw_inst: u32) -> Self {
        let rd = get_bits(raw_inst, 7, 11);
        let funct3 = get_bits(raw_inst, 12, 14);
        let rs1 = get_bits(raw_inst, 15, 19);
        let rs2 = get_bits(raw_inst, 20, 24);
        let funct7 = get_bits(raw_inst, 25, 31);

        RFormat {
            rd,
            funct3,
            rs1,
            funct7,
            rs2,
        }
    }
}

impl fmt::Display for RFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid R-format instruction: funct3: '{:b}', funct7: '{:b}'",
            self.funct3, self.funct7
        )
    }
}

struct IFormat {
    rd: usize,
    funct3: usize,
    rs1: usize,
    imm12: u32,
}
impl IFormat {
    fn new(raw_inst: u32) -> Self {
        let rd = get_bits(raw_inst, 7, 11);
        let funct3 = get_bits(raw_inst, 12, 14);
        let rs1 = get_bits(raw_inst, 15, 19);
        let imm12 = get_bits(raw_inst, 20, 31) as u32;

        IFormat {
            rd,
            funct3,
            rs1,
            imm12,
        }
    }
}
impl fmt::Display for IFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid I-format instruction: funct3: '{:b}'",
            self.funct3
        )
    }
}

enum Inst {
    ADD(RFormat),
    ADDI(IFormat),
}

impl Inst {
    fn execute(self, cpu: &mut Cpu) {
        match self {
            Inst::ADD(format) => {
                cpu.regs[format.rd] = cpu.regs[format.rs1].wrapping_add(cpu.regs[format.rs2])
            }
            Inst::ADDI(format) => {
                cpu.regs[format.rd] = cpu.regs[format.rs1].wrapping_add(format.imm12)
            }
        }
    }
}

struct Cpu {
    regs: [u32; 32],
    pc: u32,
    mem: [u8; MEMSIZE], // should wrap around
}

impl Cpu {
    fn new() -> Self {
        let mut regs = [0; 32];
        // initializes stack pointer to top of stack
        regs[2] = MEMSIZE as u32;

        Cpu {
            regs,
            pc: 0,
            mem: [0; MEMSIZE],
        }
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
                match (r_format.funct3, r_format.funct7) {
                    (0x0, 0x00) => Inst::ADD(r_format),
                    _ => return Err(Error::InvalidInstFormat(Box::new(r_format))),
                }
            }
            0b0010011 => {
                let i_format = IFormat::new(raw_inst);
                match i_format.funct3 {
                    0x0 => Inst::ADDI(i_format),
                    _ => return Err(Error::InvalidInstFormat(Box::new(i_format))),
                }
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

enum Error {
    InvalidOpcode(usize),
    InvalidInstFormat(Box<dyn fmt::Display>),
    InvalidPC,
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InvalidOpcode(opcode) => format!("invalid opcode: {:b}", opcode),
                Error::InvalidInstFormat(format) => format.to_string(),
                Error::InvalidPC => "pc bigger than mem".to_string(),
            }
        )
    }
}

// helper function to get the bits from a range [from, to] inside a u32
fn get_bits(n: u32, from: u32, to: u32) -> usize {
    // inclusive range
    let range = to - from + 1;
    // builds a binary number consisting of only ones with the len of range
    // so 3 -> 111
    let ones = (1 << range) - 1;
    // we only want to keep bits in the range
    let mask = ones << from;
    // apply mask and move matched pattern to lsb
    usize::try_from((n & mask) >> from).unwrap()
}

fn run(mut cpu: Cpu, program: Vec<u8>) -> Result<(), Error> {
    let program_cycles = program.len() / INSTSIZE_BYTES;
    cpu.load_program(program);

    for cycle in 0..program_cycles {
        cpu.emulate_cycle()?;
        cpu.dump_state(cycle);
    }

    Ok(())
}

fn read_bin() -> Vec<u8> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: ruscv <filename>");
    }
    let mut file = File::open(&args[1]).expect("valid binary input file");
    let mut program = Vec::new();
    file.read_to_end(&mut program).expect("can read binary");

    program
}

fn main() -> Result<(), Error> {
    let program = read_bin();
    let cpu = Cpu::new();
    run(cpu, program)
}
