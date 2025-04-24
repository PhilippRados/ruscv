mod cpu;
mod error;
mod inst;
mod inst_format;

use cpu::Cpu;
use error::Error;
use std::fs::File;
use std::io::Read;

struct CliArgs {
    print_debug: bool,
    filename: String,
}
impl CliArgs {
    fn new() -> Self {
        CliArgs {
            print_debug: false,
            filename: String::new(),
        }
    }
    fn parse() -> CliArgs {
        let mut cli_args = CliArgs::new();
        let mut args = std::env::args().into_iter().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-debug" => cli_args.print_debug = true,
                file if cli_args.filename.is_empty() => cli_args.filename = file.to_string(),
                _ => {
                    eprintln!("Usage: ruscv [-debug] <file>");
                    std::process::exit(1);
                }
            }
        }
        if cli_args.filename.is_empty() {
            eprintln!("Error: ruscv requires exactly one binary input file");
            eprintln!("Usage: ruscv [-debug] <file>");
            std::process::exit(1);
        }
        cli_args
    }
}

fn read_bin(path: &str) -> Vec<u8> {
    let mut file = File::open(path).expect("valid binary input file");
    let mut program = Vec::new();
    file.read_to_end(&mut program).expect("can read binary");

    program
}

fn main() -> Result<(), Error> {
    let cli_args = CliArgs::parse();
    let program = read_bin(&cli_args.filename);
    Cpu::new(cli_args.print_debug)
        .run(program)
        .and_then(|code| {
            eprintln!("Emulated program finished at exit syscall with exit-code: {code}");
            Ok(())
        })
}
