section .multiboot_header

MAGIC equ 0xe85250d6
ARCH equ 0
HEADER_LEN equ header_end - header_start
CHECKSUM equ 0x100000000 - (MAGIC + ARCH + HEADER_LEN)

header_start:
    dd MAGIC
    dd ARCH
    dd HEADER_LEN
    dd CHECKSUM
    
    dw 0
    dw 0
    dd 8
header_end:

