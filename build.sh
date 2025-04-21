#!/bin/bash

operation="$1"
filepath="$2"
dirpath=$(dirname -- "$filepath")
filename=$(basename -- "$filepath")
name="${filename%.*}"  # Extracts the file name without extension


if [[ $# -lt 2 && "$operation" != "riscv-testsuite" ]]; then
    echo "Usage: $0 {targets} <filepath>"
    echo "targets:"
    printf "\tbin:\tCreates a stripped elf binary from an asm file\n"
    printf "\trun:\tBuilds binary and runs it, from an asm file\n"
    printf "\triscv-testsuite:\tRuns official riscv testsuite, requires RISCV_TESTSUITE env var to be set to installation\n"
    printf "\tobjdump:\tPrints disassembly from .bin files\n"
    exit 1
fi

function build_bin() {
    riscv64-unknown-elf-gcc -Wl,-Ttext=0x0 -nostdlib -o "$dirpath/$name" "$filepath" -march=rv32i -mabi=ilp32
	  # strips headers off of binary and just leaves code
    riscv64-unknown-elf-objcopy -O binary "$dirpath/$name" "$dirpath/$name.bin"
}

case "$operation" in
    bin)
        build_bin
        ;;
    run)
        build_bin
        cargo run --release -- "$dirpath/$name.bin"
        ;;
    riscv-testsuite)
      if [ -z "$RISCV_TESTSUITE" ]; then
        echo "RISCV_TESTSUITE is not set"
        exit 1
      fi

      cargo b --release
      for test in "$RISCV_TESTSUITE"/isa/rv32ui-*; do
        if [[ $test != *.dump ]]; then
          test_base=$(basename -- "$test")
          # echo to stderr
          >&2 echo "Testing $test_base..."
          riscv64-unknown-elf-objcopy -O binary $test "tests/$test_base.bin"
          ./target/release/ruscv "tests/$test_base.bin"
        fi
      done
      ;;
    objdump)
        riscv64-unknown-elf-objdump -D "$filepath" -march=rv32i -mabi=ilp32
        ;;
    *)
        echo "Invalid operation: $operation"
        exit 1
        ;;
esac
