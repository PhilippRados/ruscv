.global _start
_start:
  addi x20, x0, 1
  addi x21, x0, 1
  beq x20,x21, skip
  addi x21, x21, 1
skip:
  addi x20, x20, -3
