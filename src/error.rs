use std::fmt;

pub enum Error {
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
