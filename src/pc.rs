use crate::cpu::*;
use crate::error::*;

pub struct ProgramCounter(u32);
impl ProgramCounter {
    pub fn new() -> Self {
        ProgramCounter(0)
    }
    pub fn get(&self) -> u32 {
        self.0
    }
    pub fn set(&mut self, address: u32) {
        self.0 = address
    }
    // Increments the program counter and returns the pc before it was incremented.
    // Basically a poor mans i++;
    pub fn inc(&mut self) -> Result<u32, Error> {
        let pc = self.0;
        self.0 += INSTSIZE_BYTES as u32;
        if pc > MEMSIZE as u32 - INSTSIZE_BYTES as u32 {
            return Err(Error::InvalidPC(pc, MEMSIZE));
        }
        Ok(pc)
    }
}
