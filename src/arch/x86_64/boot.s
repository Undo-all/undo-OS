VGA_ADDR equ 0xb8000
STACK_SIZE equ 4096

section .rodata
gdt64:
    dq 0
.code: equ $ - gdt64
    dq (1 << 44) | (1 << 47) | (1 << 41) | (1 << 43) | (1 << 53)
.data: equ $ - gdt64
    dq (1 << 44) | (1 << 47) | (1 << 41)
.pointer:
    dw $ - gdt64 - 1
    dq gdt64

section .bss
align 4096
p4_table:
    resb 4096
p3_table:
    resb 4096
p2_table:
    resb 4096
stack_bottom:
    resb STACK_SIZE
stack_top:

section .text
bits 32

extern long_mode_start
global start
start:
    ; Set up kernel stack
    mov esp, stack_top

    ; Check multiboot support
    cmp eax, 0x36d76289
    jne .no_multiboot 

    ; Check for CPUID by attempting to flip bit 21 of the EFLAGS register.
    pushfd
    pop eax
    mov ecx, eax
    xor eax, 1 << 21 ; flip bit
    push eax
    popfd
    pushfd
    pop eax
    push ecx
    popfd ; restore EFLAGS before flip
    cmp eax, ecx
    je .no_cpuid 

    ; Check if long mode is supported
    mov eax, 0x80000000
    cpuid
    cmp eax, 0x80000001
    jb .no_long_mode 
    mov eax, 0x80000001
    cpuid
    test edx, 1 << 29 ; only set if long mode supported
    jz .no_long_mode

    ; Set up temporary page tables
    mov eax, p3_table
    or eax, 0x3 ; 0b11, present + writable
    mov [p4_table], eax

    mov eax, p2_table
    or eax, 0x3
    mov [p3_table], eax

    mov ecx, 0
.map_p2_table:
    mov eax, 0x200000
    mul ecx
    or eax, 0x83 ; 0b10000011, present + writable + huge
    mov [p2_table + ecx * 8], eax
    
    inc ecx
    cmp ecx, 512
    jne .map_p2_table

    ; Enable paging
    ; Load P4 to cr3 register
    mov eax, p4_table
    mov cr3, eax
    
    ; Enable PAE 
    mov eax, cr4
    or eax, 1 << 5
    mov cr4, eax

    ; Set long mode bit in model specific register
    mov ecx, 0xC0000080
    rdmsr
    or eax, 1 << 8
    wrmsr

    ; Enable paging in the cr0 register
    mov eax, cr0
    or eax, 1 << 31
    mov cr0, eax

    ; Check for SSE support
    mov eax, 0x1
    cpuid
    test edx, 1 << 25
    jz .no_SSE

    ; Enable SSE
    mov eax, cr0
    and ax, 0xFFFB
    or ax, 0x2
    mov cr0, eax
    mov eax, cr4
    or ax, 3 << 9
    mov cr4, eax

    ; Load 64 bit GDT
    lgdt [gdt64.pointer]
    mov ax, gdt64.data
    mov ss, ax ; stack selector
    mov ds, ax ; data selector
    mov es, ax ; extra selector

    ; Reload code selector with a far jump
    jmp gdt64.code:long_mode_start

.no_multiboot:
    mov al, 0
    jmp error

.no_cpuid:
    mov al, 1
    jmp error

.no_long_mode:
    mov al, 2
    jmp error

.no_SSE:
    mov al, 3
    jmp error

error:
    mov dword [VGA_ADDR], 0x4f524f45
    mov dword [VGA_ADDR], 0x4f3a4f52
    mov dword [VGA_ADDR], 0x4f204f20
    add al, '0'
    mov byte [VGA_ADDR + 0xA], al

    cli
.loop:
    hlt
    jmp .loop
    
