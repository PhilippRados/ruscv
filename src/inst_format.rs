use std::fmt;

// extracts inclusive range of bits from integer, can be sign- or zero-extended depending on n_type
#[macro_export]
macro_rules! get_bits {
    // defaults to zero-extension
    ($n:expr, $from:expr, $to:expr) => {
        get_bits!($n, $from, $to, usize)
    };

    ($n:expr, $from:expr, $to:expr, $n_type:ty) => {{
        // inclusive range
        let range = $to - $from + 1;
        // builds a binary number consisting of only ones with the len of range
        // so 3 -> 111
        let ones = (1 << range) - 1;
        // we only want to keep bits in the range
        let mask = ones << $from;
        // apply mask and move matched pattern to lsb
        ($n as $n_type & mask) >> $from
    }};
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
        let rd = get_bits!(raw_inst, 7, 11);
        let funct3 = get_bits!(raw_inst, 12, 14);
        let rs1 = get_bits!(raw_inst, 15, 19);
        let rs2 = get_bits!(raw_inst, 20, 24);
        let funct7 = get_bits!(raw_inst, 25, 31);

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
    pub imm: u32,
}
impl IFormat {
    pub fn new(raw_inst: u32) -> Self {
        let rd = get_bits!(raw_inst, 7, 11);
        let funct3 = get_bits!(raw_inst, 12, 14);
        let rs1 = get_bits!(raw_inst, 15, 19);
        // immediates are sign-extended!
        let imm = get_bits!(raw_inst, 20, 31, i32) as u32;

        IFormat {
            rd,
            funct3,
            rs1,
            imm,
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

pub struct SFormat {
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
}
impl SFormat {
    pub fn new(raw_inst: u32) -> Self {
        let imm_lo = get_bits!(raw_inst, 7, 11);
        let funct3 = get_bits!(raw_inst, 12, 14);
        let rs1 = get_bits!(raw_inst, 15, 19);
        let rs2 = get_bits!(raw_inst, 20, 24);
        let imm_hi = get_bits!(raw_inst, 25, 31);
        let imm = ((imm_hi << 5) | imm_lo) as i32 as u32;

        SFormat {
            funct3,
            rs1,
            rs2,
            imm,
        }
    }
}
impl fmt::Display for SFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid I-format instruction: funct3: '{:b}'",
            self.funct3
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn get_bits_base() {
        assert_eq!(0b1101, get_bits!(0b111101101, 0, 3));
        assert_eq!(0b110, get_bits!(0b111101101, 4, 6));
    }

    #[test]
    fn get_bits_sign_extended() {
        let n: usize = 0b11111000000100000000111110010011;
        assert_eq!(-127, get_bits!(n, 20, 31, i32));
        assert_eq!(0b011, get_bits!(n, 10, 13, i32));
    }
}
