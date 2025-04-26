use crate::memory::*;

pub struct Registers([u32; 32]);
impl Registers {
    pub fn new() -> Self {
        let mut regs = Registers([0; 32]);
        // initializes stack pointer to top of stack
        regs.0[2] = MEMSIZE as u32;
        regs
    }
    pub fn read(&self, reg_idx: usize) -> u32 {
        assert!(reg_idx < 32, "rv32i only has 32 registers");
        if reg_idx == 0 {
            0
        } else {
            self.0[reg_idx]
        }
    }
    pub fn write(&mut self, reg_idx: usize, value: u32) {
        assert!(reg_idx < 32, "rv32i only has 32 registers");
        if reg_idx == 0 {
            return;
        }

        self.0[reg_idx] = value;
    }
}
