section .text
bits 64

extern kmain
global long_mode_start
long_mode_start:
    call kmain 
 
    cli
.loop:
    hlt
    jmp .loop

