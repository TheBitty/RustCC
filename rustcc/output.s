.section __TEXT,__text,regular,pure_instructions
.globl _main
.p2align 4, 0x90
_main: 
    push %rbp
    mov %rsp, %rbp
    sub $48, %rsp
    mov $98, %rax
    mov %rax, -8(%rbp)
    mov $10, %rax
    mov %rax, -16(%rbp)
    mov $20, %rax
    mov %rax, -24(%rbp)
    mov $89, %rax
    mov %rax, -32(%rbp)
    mov -24(%rbp), %rax
    push %rax
    mov -16(%rbp), %rax
    pop %rcx
    add %rcx, %rax
    mov %rax, -40(%rbp)
    mov $2, %rax
    push %rax
    mov $2, %rax
    push %rax
    mov -40(%rbp), %rax
    pop %rcx
    imul %rcx, %rax
    pop %rcx
    cqo
    idiv %rcx
    mov %rbp, %rsp
    pop %rbp
    ret
    mov %rbp, %rsp
    pop %rbp
    ret
