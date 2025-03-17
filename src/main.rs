mod cpu;
mod error;
mod inst;
mod inst_format;

use cpu::Cpu;
use error::Error;
use std::fs::File;
use std::io::Read;

fn read_bin() -> Vec<u8> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 2 {
        panic!("Usage: ruscv <filename>");
    }
    let mut file = File::open(&args[1]).expect("valid binary input file");
    let mut program = Vec::new();
    file.read_to_end(&mut program).expect("can read binary");

    program
}

fn main() -> Result<(), Error> {
    let program = read_bin();
    Cpu::new().run(program)
}
