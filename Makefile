run:
	qemu-system-x86_64 -drive format=raw,file=/root/myos/os_x86_64/target/x86_64-blog_os/debug/bootimage-os_x86_64.bin

console:
	qemu-system-x86_64 -drive format=raw,file=/root/myos/os_x86_64/target/x86_64-blog_os/debug/bootimage-os_x86_64.bin -nographic -serial mon:stdio

debug:
	qemu-system-x86_64 -drive format=raw,file=/root/myos/os_x86_64/target/x86_64-blog_os/debug/bootimage-os_x86_64.bin -s -S

build:
	cargo bootimage

clean:
	cargo clean
