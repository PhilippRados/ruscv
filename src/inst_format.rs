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
        let imm_lo = get_bits!(raw_inst, 7, 11, i32);
        let funct3 = get_bits!(raw_inst, 12, 14);
        let rs1 = get_bits!(raw_inst, 15, 19);
        let rs2 = get_bits!(raw_inst, 20, 24);
        let imm_hi = get_bits!(raw_inst, 25, 31, i32);
        let imm = ((imm_hi << 5) | imm_lo) as u32;

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
            "invalid S-format instruction: funct3: '{:b}'",
            self.funct3
        )
    }
}

pub struct BFormat {
    pub funct3: usize,
    pub rs1: usize,
    pub rs2: usize,
    pub imm: u32,
}
impl BFormat {
    // RISC-V Spec: 2.3
    // The only difference between the S and B formats is that the 12-bit immediate field is used to encode
    // branch offsets in multiples of 2 in the B format. Instead of shifting all bits in the instruction-encoded
    // immediate left by one in hardware as is conventionally done, the middle bits (imm[10:1]) and sign bit
    // stay in fixed positions, while the lowest bit in S format (inst[7]) encodes a high-order bit in B format.
    pub fn new(raw_inst: u32) -> Self {
        let imm_lo = get_bits!(raw_inst, 8, 11, i32);
        let imm_11th_bit = get_bits!(raw_inst, 7, 7, i32);
        let funct3 = get_bits!(raw_inst, 12, 14);
        let rs1 = get_bits!(raw_inst, 15, 19);
        let rs2 = get_bits!(raw_inst, 20, 24);
        let imm_hi = get_bits!(raw_inst, 25, 30, i32);
        let imm_12th_bit = get_bits!(raw_inst, 31, 31, i32);

        let imm =
            ((imm_12th_bit << 12) | (imm_11th_bit << 11) | (imm_hi << 5) | (imm_lo << 1)) as u32;

        BFormat {
            funct3,
            rs1,
            rs2,
            imm,
        }
    }
}
impl fmt::Display for BFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid B-format instruction: funct3: '{:b}'",
            self.funct3
        )
    }
}

pub struct JFormat {
    pub rd: usize,
    pub imm: u32,
}
impl JFormat {
    pub fn new(raw_inst: u32) -> Self {
        let rd = get_bits!(raw_inst, 7, 11);
        let imm_hi = get_bits!(raw_inst, 12, 19, i32);
        let imm_11th_bit = get_bits!(raw_inst, 20, 20, i32);
        let imm_lo = get_bits!(raw_inst, 21, 30, i32);
        let imm_20th_bit = get_bits!(raw_inst, 31, 31, i32);
        let imm =
            ((imm_20th_bit << 20) | (imm_hi << 12) | (imm_11th_bit << 11) | (imm_lo << 1)) as u32;

        JFormat { rd, imm }
    }
}

pub struct UFormat {
    pub rd: usize,
    pub imm: u32,
}
impl UFormat {
    pub fn new(raw_inst: u32) -> Self {
        let rd = get_bits!(raw_inst, 7, 11);
        let imm = get_bits!(raw_inst, 12, 31, i32) as u32;

        UFormat { rd, imm }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn parse_cond_br_imm() {
        // bge x0, x0, -12
        assert_eq!(
            BFormat::new(0b1111_1110_0000_0000_0101_1010_1110_0011).imm as i32,
            -12
        );
    }
}
