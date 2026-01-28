arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
grub_platform := i386-pc
linker_script := src/arch/$(arch)/linker.ld
grub_cfg := src/arch/$(arch)/grub.cfg
asm_src_files := $(wildcard src/arch/$(arch)/*.asm)
asm_obj_files := $(patsubst src/arch/$(arch)/%.asm, \
								 build/arch/$(arch)/%.o, $(asm_src_files))

target ?= $(arch)-tiny_os
rust_build_mode := debug
rust_os := target/$(target)/$(rust_build_mode)/libtiny_os.a

user_startup_program := tetris
user_program_location := target/$(target)/$(rust_build_mode)
user_program_path := $(user_program_location)/$(user_startup_program)

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
		@rm -r build

run: $(iso)
	# @qemu-system-x86_64 -device isa-fdc -d int -cdrom $(iso) -m 1G
	# @qemu-system-x86_64 -device isa-fdc -cdrom $(iso) -m 1G
	@qemu-system-x86_64 -accel kvm -cpu host -device isa-fdc -cdrom $(iso) -m 1G

iso: $(iso)

$(iso): $(kernel) $(grub_cfg) usermode_program
	@mkdir -p build/isofiles/boot/grub
	@cp $(user_program_path) build/arch/$(arch)/usermode_program
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub/grub.cfg
	@grub-mkrescue -o $(iso) build/isofiles -d /usr/lib/grub/$(grub_platform)
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(asm_obj_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(asm_obj_files) $(rust_os)

usermode_program:
	@RUST_TARGET_PATH=$(shell pwd) cargo build -p tetris --target $(target) $(if $(filter release,$(rust_build_mode)),--release)

kernel:
	@RUST_TARGET_PATH=$(shell pwd) cargo build --target $(target) $(if $(filter release,$(rust_build_mode)),--release)

build/arch/$(arch)/%.o: src/arch/$(arch)/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@
