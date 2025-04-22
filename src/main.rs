mod cpu;
mod error;
mod inst;
mod inst_format;

use cpu::Cpu;
use error::Error;
use std::fs::File;
use std::io::Read;

fn read_bin(path: &str) -> Vec<u8> {
    let mut file = File::open(path).expect("valid binary input file");
    let mut program = Vec::new();
    file.read_to_end(&mut program).expect("can read binary");

    program
}

fn main() -> Result<(), Error> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        eprintln!("Error: Emulator requires exactly one argument");
        eprintln!("Usage: ruscv <file>");
        std::process::exit(1);
    }

    let program = read_bin(&args[1]);
    Cpu::new().run(program).and_then(|code| {
        eprintln!("Emulated program finished at exit syscall with exit-code: {code}");
        Ok(())
    })
}
