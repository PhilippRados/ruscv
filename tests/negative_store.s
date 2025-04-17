_start:
  addi x28, x0, 60
  # make sure not to overwrite program memory so use higher address like 256
  addi x22, x0, 261
  sw x28, -5(x22)
  addi x27, x0, 256
  lw x30, 0(x27)
