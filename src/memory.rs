use crate::inst::*;

// Don't want to use too much memory for emulator
pub const MEMSIZE: usize = 1024 * 128;
// Start address of dram section
// pub const MEM_START: u32 = 0x8000_0000;

#[derive(Clone)]
pub enum Size {
    Byte = 1,
    HalfWord = 2,
    Word = 4,
}
impl From<LoadIInst> for Size {
    fn from(value: LoadIInst) -> Self {
        match value {
            LoadIInst::LB | LoadIInst::LBU => Size::Byte,
            LoadIInst::LH | LoadIInst::LHU => Size::HalfWord,
            LoadIInst::LW => Size::Word,
        }
    }
}

impl From<SInst> for Size {
    fn from(value: SInst) -> Self {
        match value {
            SInst::SB => Size::Byte,
            SInst::SH => Size::HalfWord,
            SInst::SW => Size::Word,
        }
    }
}

macro_rules! read_mem {
    ($ty:ty,$mem:expr,$from:expr,$to:expr) => {
        <$ty>::from_le_bytes($mem[$from as usize..$to as usize].try_into().unwrap()) as u32
    };
}
pub struct Memory([u8; MEMSIZE]);
impl Memory {
    pub fn new() -> Self {
        Memory([0; MEMSIZE])
    }
    pub fn read(&self, size: Size, from: u32, is_unsigned: bool) -> u32 {
        let to = from + size.clone() as u32;
        match (size, is_unsigned) {
            (Size::Byte, true) => read_mem!(u8, self.0, from, to),
            (Size::HalfWord, true) => read_mem!(u16, self.0, from, to),
            (Size::Byte, false) => read_mem!(i8, self.0, from, to),
            (Size::HalfWord, false) => read_mem!(i16, self.0, from, to),
            (Size::Word, _) => read_mem!(u32, self.0, from, to),
        }
    }
    pub fn write(&mut self, size: Size, address: u32, value: u32) {
        let slice = value.to_le_bytes();
        let address = address as usize;
        match size {
            Size::Byte => self.0[address..address + size as usize].copy_from_slice(&slice[0..1]),
            Size::HalfWord => {
                self.0[address..address + size as usize].copy_from_slice(&slice[0..2])
            }
            Size::Word => self.0[address..address + size as usize].copy_from_slice(&slice[0..4]),
        }
    }

    // loads program to start of the memory
    pub fn load_program(&mut self, mut program: Vec<u8>) {
        program.resize_with(MEMSIZE, || 0);
        self.0 = program.as_slice().try_into().unwrap()
    }
}
