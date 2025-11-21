# Tiny OS

This is a tiny OS to learn how operating systems work.

## Requirements

- Make
- NASM
- grub-mkrescue
    - xorriso
    - grub-pc-bin (some installations need this to work)
- qemu (optional for running)

## Bootloader

The bootloader will be written for 64-bit.

It uses multiboot boot using something like grub.

afterwards the switch to 64-bit mode happens.

Help from [here](https://os.phil-opp.com/entering-longmode/)

## Pages
https://os.phil-opp.com/page-tables/

## current progress
https://os.phil-opp.com/remap-the-kernel/
