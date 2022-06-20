        # borrowed from xv6-riscv
	# qemu -kernel loads the kernel at 0x80000000
        # and causes each CPU to jump there.
        # kernel.ld causes the following code to
        # be placed at 0x80000000.
        .section .entry.text
        .global _entry
_entry:
	# set up a stack for C.
        # c_stack is declared in start_rust.c,
        # with a 4096-byte stack per CPU.
        # sp = c_stack + (hartid * 4096)
        la sp, c_stack
        li a0, 1024*4
	csrr a1, mhartid
        addi a1, a1, 1
        mul a0, a0, a1
        add sp, sp, a0
	# jump to start_rust() in start_rust.c
        call start_rust
spin:
        j spin
