run: build
	qemu-system-x86_64 -drive format=raw,file=/root/myos/os_x86_64/target/x86_64-blog_os/debug/bootimage-os_x86_64.bin

console: build
	qemu-system-x86_64 -drive format=raw,file=/root/myos/os_x86_64/target/x86_64-blog_os/debug/bootimage-os_x86_64.bin -nographic -serial mon:stdio

debug: build
	qemu-system-x86_64 -drive format=raw,file=/root/myos/os_x86_64/target/x86_64-blog_os/debug/bootimage-os_x86_64.bin -s -S

build: user_programs
	cargo bootimage

user_programs:
	chmod +x build_user_programs.sh
	./build_user_programs.sh

clean:
	cargo clean
	rm -rf target/user_programs
