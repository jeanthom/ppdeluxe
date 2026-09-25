OUTPUT_ARCH( "arm" )

ENTRY( _start )

MEMORY
{
    dram (wxa) : ORIGIN = 0x10000000, LENGTH = 0x2000000

    iram (wxa) : ORIGIN = 0x40000000, LENGTH = 0x18000
}

PHDRS
{
    text PT_LOAD;
    data PT_LOAD;
    bss PT_LOAD;
}

SECTIONS {

    .text ORIGIN(iram) : {
        *(.text.init)
        *(.text .text.*)
    } >iram AT>dram :text

    .rodata : {
        *(.rodata .rodata.*)
        PROVIDE(_kernel_rom_end = .);
    } >iram AT>dram :text

    .data : {
        *(.sdata .sdata.*) *(.data .data.*)
    } >iram AT>dram :data

    PROVIDE(_iram_end = .);

    .stack (NOLOAD) : {
        . = ALIGN(8);
        _stack_start = .;
        . += 0x2000;
        _stack_end = .;
    } >dram

    .bss : {
        PROVIDE(_bss_start = .);
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        PROVIDE(_bss_end = .);
    } >dram :bss

    PROVIDE(_MEM_START = _bss_end);
    PROVIDE(_MEM_END = _MEM_START + LENGTH(dram) - (_bss_end - _bss_start));

    /DISCARD/ : {
        *(.ARM.exidx* .ARM.extab*)
    }
}
