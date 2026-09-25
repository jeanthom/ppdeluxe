.section .text.init

.macro lcd_backlight on
	ldr r0, =0x6000d824
.if \on == 1
	ldr r1, =0x101
.else
	ldr r1, =0x100
.endif
	lsl r1, r1, #3
	str r1, [r0]
.endm

.global _start
_start:
    /* based heavily on code from rockbox, which in turn was based on code from ipodlinux */
    /* turn off interrupts */
    msr cpsr_c, #0xdf

    /* we're not using the coprocessor for now, so disable it completely */
    // todo: actually put it to sleep and don't waste cycles like this

	ldr r0, =0x60000000 // load constant
	ldrb r0, [r0] // load value of PROC_ID
	cmp r0, #0x55
	beq cpu
cop:
	nop
	b cop
cpu:

    lcd_backlight 0

    /* we must Move the code from where it is to iram */

    bic r1, pc, #0xff /* top part of pc address */
    mov r2, #0x40000000 /* r2 contains iram address */
    ldr r3, =_iram_end  /* r3 contains address to where we stop loading */

1:  /* there's probably a better way to do this, but i'm not very skilled in arm assembly */
    ldr r4, [r1], #4 /* load dram data */
    str r4, [r2], #4 /* store data in iram */
    cmp r3, r2 /* if _iram_end >= iram_ptr (r2), loop */
    bhi 1b

    lcd_backlight 1

    ldr pc, =reloc_start /* start executing from iram */

reloc_start:
    lcd_backlight 1
    ldr sp, =_stack_end

    /* zero out bss */
    ldr r1, =_bss_start
    ldr r2, =_bss_end
    mov r3, 0
1:
    str r3, [r1], #4
    cmp r2, r1
    bhi 1b

    bl _ipod_main

1:
    nop
    b 1b