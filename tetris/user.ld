ENTRY(tetris_user_main)

SECTIONS
  {
      /* start address must match USERMODE_CODE_START in your kernel */
      . = 0x8000000;  

      .text : {
        *(.text*)      /* all .text sections */
        *(.rodata*)    /* read-only constants */
    }

    .data : {
        *(.data*)      /* initialized data */
    }

    .bss : {
        *(.bss*)       /* uninitialized data */
        *(COMMON)
    }
}
