ifndef SDCARD_SIZE_MB
	#Default size if 8Mib
	SDCARD_SIZE_KB=8
endif

CARGO_BUILD_FLAGS = --release


apps = loop10 hello_world get_pid sys_wait4 sys_brk sys_kill \
	  	forkboom signal_chld times nanosleep openat pipe dup \
		mkdirat chdir get_dirents sys_clone execve shell read filelink contest_test

all: os.bin

qemu:
	make os.bin
	qemu-system-riscv64 -M sifive_u -smp 5 \
		-bios bootloader/fw_jump.bin \
		-sd fat32.img \
		-device loader,file=os.bin,addr=0x80200000 \
		-nographic

user_apps:
	@cat userenv/cargo.toml.template > userenv/Cargo.toml
	@for x in $(apps); do \
		echo "\n[[bin]]\n" >> userenv/Cargo.toml; \
		echo "name = \"$$x\"\n" >> userenv/Cargo.toml; \
		echo "path = \"src/$$x.rs\"\n" >> userenv/Cargo.toml; \
	done
	@[ -e src/user/bin ] || mkdir src/user/bin
	@cd userenv && cargo build
	@for x in $(apps); do \
		mv -f userenv/target/riscv64gc-unknown-none-elf/debug/$$x src/user/bin/$$x; \
	done

os.bin: user_apps
	@cargo build $(CARGO_BUILD_FLAGS)
	@if which rust-objcopy ; then \
		rust-objcopy target/riscv64gc-unknown-none-elf/release/os -O binary os.bin; \
	elif which riscv-objcopy; then \
		riscv-objcopy target/riscv64gc-unknown-none-elf/release/os -O binary os.bin; \
	elif which objcopy ; then \
		objcopy target/riscv64gc-unknown-none-elf/release/os -O binary os.bin; \
	else \
	  @echo objcopy not found; \
	fi


sdcard.raw:
		dd if=/dev/zero of=sdcard.img bs=1048576 count=$(SDCARD_SIZE_KB)

sdcard.part: sdcard.raw
		@sgdisk --clear --set-alignment=2 -g \
		  --new=1:34:2047 --change-name=1:spl --typecode=1:5B193300-FC78-40CD-8002-E86C45580B47 \
			--new=2:2048:10239 --change-name=2:u-boot_sbi --typecode=2:2E54B353-1271-4842-806F-E436D6AF6985 \
			--new=3:10240: --change-name=3:rootfs --typecode=3:C12A7328-F81F-11D2-BA4B-00A0C93EC93B  \
			sdcard.img

fat32.img:
		dd if=/dev/zero of=fat32.img bs=1M count=512
		mkfs.fat -F 32 fat32.img

rootfs: sdcard.part fat32.img os.bin
		sudo mount fat32.img mnt
		sudo cp os.bin mnt/os
		sudo umount mnt
		dd if=fat32.img of=sdcard.img bs=512 seek=10240 conv=notrunc

sdcard-unleashed: sdcard.part rootfs
		sudo dd if=bootloader/u-boot-spl.bin of=sdcard.img bs=512 seek=34 conv=notrunc
		sudo dd if=bootloader/fw_payload_u-boot_unleashed.bin of=sdcard.img bs=512 seek=2048 conv=notrunc

qemu-unleashed: sdcard-unleashed
		qemu-system-riscv64 -M sifive_u \
			-bios bootloader/fw_payload_u-boot_unleashed.bin \
			-drive file=sdcard.img,if=sd,format=raw \
			-nographic
qemu-unleashed-jump: os.bin
		qemu-system-riscv64 -M sifive_u\
			-bios bootloader/fw_jump.bin \
			-drive file=sdcard.img,if=sd,format=raw \
			-device loader,file=os.bin,addr=0x80200000 \
			-nographic
gdb: os.bin
		qemu-system-riscv64 -M sifive_u\
			-bios bootloader/fw_jump.bin \
			-drive file=sdcard.img,if=sd,format=raw \
			-kernel target/riscv64gc-unknown-none-elf/debug/os \
			-S -s -nographic
qemu-unleashed-rustsbi: os.bin
		qemu-system-riscv64 --machine virt \
			-bios bootloader/rustsbi-qemu.bin \
			-device loader,file=os.bin,addr=0x80200000 \
			-nographic

clean:
		@rm -f os.bin
		@rm -f src/user/bin/*
		@cargo clean
