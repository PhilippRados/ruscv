.global _start
_start:
  addi x20,x0, -1
  addi x21,x0, 1
  blt x20,x21, end
  addi x20,x0,100
  addi x21,x0,100
end:
