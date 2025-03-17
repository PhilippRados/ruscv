use std::fmt;

// helper function to get the bits from a range [from, to] inside a u32
pub fn get_bits(n: u32, from: u32, to: u32) -> usize {
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

pub struct RFormat {
    pub rd: usize,
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub funct7: usize,
}
impl RFormat {
    pub fn new(raw_inst: u32) -> Self {
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

pub struct IFormat {
    pub rd: usize,
    pub funct3: usize,
    pub rs1: usize,
    pub imm12: u32,
}
impl IFormat {
    pub fn new(raw_inst: u32) -> Self {
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
