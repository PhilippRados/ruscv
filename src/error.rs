use std::fmt;

pub enum Error {
    InvalidOpcode(usize),
    InvalidInstFormat(Box<dyn fmt::Display>),
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
                Error::InvalidInstFormat(format) => format.to_string(),
                Error::InvalidPC(pc, memsize) => format!(
                    "Program counter (pc: {pc}) bigger than than memory (memsize: {memsize}B)"
                ),
                Error::EndOfInstructions =>
                    "Program ran out of instructions! Use exit syscall to terminate gracefully."
                        .to_string(),
            }
        )
    }
}
