use std::fmt;

pub enum Error {
    InvalidOpcode(usize),
    InvalidInstFormat(String),
    InvalidPC(usize, usize),
    EndOfInstructions,
}
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Error::InvalidOpcode(opcode) => format!("invalid opcode: {:b}", opcode),
                Error::InvalidInstFormat(format) => format.clone(),
                Error::InvalidPC(pc, memsize) => format!(
                    "program counter (pc: {pc}) bigger than than memory (memsize: {memsize}B)"
                ),
                Error::EndOfInstructions =>
                    "program ran out of instructions! Use exit syscall to terminate gracefully."
                        .to_string(),
            }
        )
    }
}
