OUTPUT_ARCH(riscv)
ENTRY(_binary_kernel_bin_start)

SECTIONS
{
    . = 0x80200000;
    .text : {
        *(.text)
    }
}
