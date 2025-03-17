#!/bin/bash

if [[ $# -lt 2 ]]; then
    echo "Usage: $0 {bin|run|objdump} <filepath>"
    exit 1
fi

operation="$1"
filepath="$2"
dirpath=$(dirname -- "$filepath")
filename=$(basename -- "$filepath")
name="${filename%.*}"  # Extracts the file name without extension


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
        cargo run -- "$dirpath/$name.bin"
        ;;
    objdump)
        riscv64-unknown-elf-objdump -s "$filepath" -march=rv32i -mabi=ilp32
        ;;
    *)
        echo "Invalid operation: $operation"
        exit 1
        ;;
esac
