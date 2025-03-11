.section __TEXT,__text,regular,pure_instructions
.globl _main
.p2align 4, 0x90
_main: 
    push %rbp
    mov %rsp, %rbp
    sub $24, %rsp
    mov $42, %rax
    mov %rax, -8(%rbp)
    mov $10, %rax
    mov %rax, -16(%rbp)
    mov -16(%rbp), %rax
    push %rax
    mov -8(%rbp), %rax
    pop %rcx
    add %rcx, %rax
    mov %rbp, %rsp
    pop %rbp
    ret
    mov %rbp, %rsp
    pop %rbp
    ret
