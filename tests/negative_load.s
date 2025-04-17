.global _start
_start:
  addi x28, x0, 60
  sw x28, 20(x0)
  addi x27, x0, 21
  lw x30, -1(x27)
