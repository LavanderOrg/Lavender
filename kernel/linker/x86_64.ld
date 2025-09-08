OUTPUT_FORMAT(elf64-x86-64)
ENTRY(kmain)

PHDRS
{
    text    PT_LOAD;
    rodata  PT_LOAD;
    data    PT_LOAD;
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : {
        LD_TEXT_START = .;
        *(.text .text.*)
        LD_TEXT_END = .;
    } :text

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .rodata : {
        LD_RODATA_START = .;
        *(.rodata .rodata.*)
        LD_RODATA_END = .;
    } :rodata

    . = ALIGN(CONSTANT(MAXPAGESIZE));
    .data : {
        LD_DATA_START = .;
        *(.data .data.*)

        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))
        LD_DATA_END = .;
    } :data

    .bss : {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    /DISCARD/ : {
        *(.eh_frame*)
        *(.note .note.*)
    }
}