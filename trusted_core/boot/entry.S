        # borrowed from xv6-riscv
	# qemu -kernel loads the kernel at 0x80000000
        # and causes each CPU to jump there.
        # kernel.ld causes the following code to
        # be placed at 0x80000000.
        .section .text.entry
        .global _entry
_entry:
        # set up a stack for C.
        # kernel_stack is declared in start_rust.c,
        # with a 4096-byte stack per CPU.
        # sp = kernel_stack + (hartid * 4096)
        li x1, 0
        li x2, 0
        li x3, 0
        li x4, 0
        li x5, 0
        li x6, 0
        li x7, 0
        li x8, 0
        li x9, 0
        #save a0 and a1; arguments from previous boot loader stage:
        li x12, 0
        li x13, 0
        li x14, 0
        li x15, 0
        li x16, 0
        li x17, 0
        li x18, 0
        li x19, 0
        li x20, 0
        li x21, 0
        li x22, 0
        li x23, 0
        li x24, 0
        li x25, 0
        li x26, 0
        li x27, 0
        li x28, 0
        li x29, 0
        li x30, 0
        li x31, 0
        csrw mscratch, x0

        li t0, 0
        csrs mstatus, t0
        la sp, kernel_stack
        li a0, 1024*4
        csrr a1, mhartid
        addi a1, a1, 1
        mul a0, a0, a1
        add sp, sp, a0
        # jump to start_rust() in start_rust.c
        call start_rust
spin:
        j spin
