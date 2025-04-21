.global _start
_start:
  addi x27, x0, 60
  sw x27, 64(x0)
  lw x30, 64(x0)
  lh x29, 64(x0)
  lb x28, 64(x0)
