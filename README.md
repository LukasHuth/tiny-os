# Tiny OS

This is a tiny OS to learn how operating systems work.

## Bootloader

The bootloader will be written for 64-bit 

### Bootloader Roadmap

#### 16-bit real mode
- BIOS loads our 512-byte boot sector at 0x7C00
- Set up GDT data structures in memory
- Enable A20 line
- Switch to protected mode

#### 32-bit protected mode
- Enable PAE
- Load page tables
- Enable long mode in EFER
- Switch to long mode

#### 64-bit long mode
- Print something using a simple routine
- Halt
