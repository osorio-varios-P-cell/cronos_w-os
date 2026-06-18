OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(i386:x86-64)
ENTRY(_start)

PHDRS
{
    text PT_LOAD FLAGS(5);
    data PT_LOAD FLAGS(6);
}

SECTIONS
{
    . = 0xffffffff80000000;

    .text : ALIGN(4K) {
        *(.text .text.*)
    } :text

    .rodata : ALIGN(4K) {
        *(.rodata .rodata.*)
        KEEP(*(.requests_start_marker))
        KEEP(*(.requests))
        KEEP(*(.requests_end_marker))
    } :text

    .data : ALIGN(4K) {
        *(.data .data.*)
        *(.got .got.*)
    } :data

    .bss : ALIGN(4K) {
        *(.bss .bss.*)
        *(COMMON)
    } :data

    . = ALIGN(4K);
    __heap_start = .;
    . = . + 16M;
    __heap_end = .;

    /DISCARD/ : {
        *(.eh_frame)
        *(.note .note.*)
        *(.dynsym)
        *(.gnu.hash)
        *(.hash)
        *(.dynstr)
        *(.rela.dyn)
        *(.rela.*)
        *(.dynamic)
    }
}
