# ruscv
### A RISC-V (rv32i) emulator.<br>
This is a small emulator that implements the basic rv32i isa based on the references in [docs](docs).
It passes all the rv32ui-p* test cases in the official [riscv-tests](https://github.com/riscv-software-src/riscv-tests).

## Installation
I haven't published it to crates.io since this is just a toy project. You can still install it through git though:
```bash
$ cargo install --git https://github.com/PhilippRados/ruscv.git
```
Or just clone this repo and build from source.

## Usage
The emulator expects a raw binary file and starts executing it at address 0.
The emulator stops when it encounters an exit syscall (ecall with a7 = 93) or when it runs out of instructions (ie. inst is all zeros). 
```bash
$ ruscv <file.bin> # runs binary file and prints exit code and last emulator state.
$ ruscv <file.bin> -debug # adds additional debug info and prints emulator state after each cycle.
```
The targets in [build.sh](build.sh) allow to run the emulator from assembly files.
This requires an installation of the riscv64-unknown-elf-* toolchain to be installed in your $PATH.
```bash
$ ./build.sh run tests/fibs.s
```

## Tests
The tests also require the riscv-toolchain to be installed.<br>
Unit tests can be run using `cargo t` which then also tests the assembly files in the [tests](tests/) folder.<br>
Additionally if you have the [riscv-tests](https://github.com/riscv-software-src/riscv-tests) installed then you can run them like this:
```bash
$ RISCV_TESTSUITE=<path-to-folder> ./build.sh riscv-testsuite
```
This requires the environment variable `RISCV_TESTSUITE` to point to the installation path of the testsuite.

## Resources
These resources helped me during development (aside from the [docs](docs/)).
- Encode/Decode binary instructions: https://luplab.gitlab.io/rvcodecjs/
- RISC-V instructions explained: https://projectf.io/posts/riscv-cheat-sheet/
- Online emulator to test against: https://www.cs.cornell.edu/courses/cs3410/2019sp/riscv/interpreter/

## Todo
- [ ] It would be nice to have some working syscalls to interact with the outside world.
- [ ] Run some bigger real-world programs (linux seems to be the thing people like to try but that seems out of scope for this small project).
