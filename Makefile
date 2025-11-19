arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
grub_platform := i386-pc

linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
asm_src_files := $(wildcard src/arch/$(arch)/*.asm)
asm_obj_files := $(patsubst src/arch/$(arch)/%.asm, \
								 build/arch/$(arch)/%.o, $(asm_src_files))

.PHONY: all clean run iso

all: $(kernel)

clean:
		@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub/grub.cfg
	@grub-mkrescue -o $(iso) build/isofiles -d /usr/lib/grub/$(grub_platform)
	@rm -r build/isofiles

$(kernel): $(asm_obj_files) $(linker_script)
	@ld -n -T $(linker_script) -o $(kernel) $(asm_obj_files)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
