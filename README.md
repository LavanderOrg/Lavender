# Lavender

**Rust OS built around an UNIX-Like kernel**

Lavender is yet another original OS made using Rust and using the [Limine bootloader](https://github.com/limine-bootloader/limine)
It currently only supports x86_64 with UEFI and CPUID.

## Prerequesites

- Install QEMU (See: https://www.qemu.org/download/)

## Instructions

To build:

```bash
make build
```

To run the kernel using QEMU:
*This will download Limine and setup the ISO to boot from.*

```bash
make run
```

<img width="1947" height="1064" alt="image" src="https://github.com/user-attachments/assets/38532a9a-890c-4ebc-baeb-b02ce9f32d01" />
