KERNEL_NAME = os
KERNEL_BUILD = release
CARGO_BUILD_FLAGS = --$(KERNEL_BUILD) --offline
TRIPLE = riscv64imac-unknown-none-elf

apps = loop10 hello_world get_pid sys_wait4 sys_brk sys_kill \
	  	forkboom signal_chld times nanosleep openat pipe dup \
		mkdirat chdir get_dirents sys_clone execve shell read filelink contest_test

run: toolchain qemu

all: toolchain $(KERNEL_NAME).bin

toolchain:
	@rustup target add $(TRIPLE)

qemu: $(KERNEL_NAME).bin
	qemu-system-riscv64 -M sifive_u -smp 5 \
		-bios bootloader/fw_jump.bin \
		-sd fat32.img \
		-device loader,file=$(KERNEL_NAME).bin,addr=0x80200000 \
		-nographic

user_apps:
	@cat userenv/cargo.toml.template > userenv/Cargo.toml
	@for x in $(apps); do \
		echo "\n[[bin]]\n" >> userenv/Cargo.toml; \
		echo "name = \"$$x\"\n" >> userenv/Cargo.toml; \
		echo "path = \"src/$$x.rs\"\n" >> userenv/Cargo.toml; \
	done
	@cd userenv && cargo build

userenv/target/$(TRIPLE)/debug/%: userenv/src/
	@make user_apps

src/user/bin/%: userenv/target/$(TRIPLE)/debug/%
	@mv -f $^ src/user/bin

$(KERNEL_NAME).bin: src/ Cargo.toml Makefile src/user/bin/contest_test src/user/bin/shell
	@cargo build $(CARGO_BUILD_FLAGS)
	@if which rust-objcopy ; then \
		rust-objcopy target/$(TRIPLE)/$(KERNEL_BUILD)/$(KERNEL_NAME) -O binary $(KERNEL_NAME).bin; \
	elif which $(TRIPLE)-objcopy; then \
		$(TRIPLE)-objcopy target/$(TRIPLE)/$(KERNEL_BUILD)/$(KERNEL_NAME) -O binary $(KERNEL_NAME).bin; \
	elif which objcopy ; then \
		objcopy target/$(TRIPLE)/$(KERNEL_BUILD)/$(KERNEL_NAME) -O binary $(KERNEL_NAME).bin; \
	else \
	  echo objcopy not found; \
	fi

clean:
		@rm -f $(KERNALE_NAME).bin
		@rm -f src/user/bin/*
		@cargo clean

.PHONY: clean user_apps qemu run all
