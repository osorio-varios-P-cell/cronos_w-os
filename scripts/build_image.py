#!/usr/bin/env python3
"""Build a bootable GPT disk image for CRONOS OS using manual GPT and FATtools."""
import os
import struct
import subprocess

WORK = r"F:\cronos_w-os"
KERNEL = os.path.join(WORK, "target", "x86_64-unknown-none", "release", "cronos_w_os")
LIMINE_CFG = os.path.join(WORK, "limine.cfg")
LIMINE_DIR = os.path.join(WORK, "limine-tools", "limine-binary")
LIMINE_EXE = os.path.join(LIMINE_DIR, "limine-tool-windows-x86", "limine.exe")
BOOTX64 = os.path.join(LIMINE_DIR, "BOOTX64.EFI")
OUTPUT = os.path.join(WORK, "cronos_w-os.img")

SECTOR = 512
GPT_ENTRIES_LBA = 2
GPT_ENTRIES_COUNT = 128
GPT_ENTRY_SIZE = 128
HEADER_SIZE = 92

def main():
    # Parameters
    img_mb = 64
    img_size = img_mb * 1024 * 1024
    total_lba = img_size // SECTOR
    esp_start = 34
    # ESP partition size: 16MB = 16 * 1024 * 1024 / 512 = 32768 LBAs
    # Further reduced to avoid GPT LBA overflow
    esp_lba = 16 * 1024 * 1024 // SECTOR

    # ESP partition offset and size in bytes
    esp_offset = esp_start * SECTOR
    esp_bytes = esp_lba * SECTOR

    print(f"[BUILD] Image: {img_mb}MB, ESP: LBA {esp_start}-{esp_start+esp_lba-1} ({esp_bytes//1024//1024}MB)")

    # ── Create empty image ──
    with open(OUTPUT, "wb") as f:
        f.truncate(img_size)

    # ── Protective MBR (LBA 0) ──
    mbr = bytearray(SECTOR)
    mbr[0x1BE] = 0x00
    mbr[0x1BF:0x1C2] = b"\x00\x02\x00"
    mbr[0x1C2] = 0xEE
    mbr[0x1C3:0x1C6] = b"\xFF\xFF\xFF"
    struct.pack_into("<I", mbr, 0x1C6, 1)
    struct.pack_into("<I", mbr, 0x1CA, min(total_lba - 1, 0xFFFFFFFF))
    mbr[0x1FE:0x200] = b"\x55\xAA"
    with open(OUTPUT, "r+b") as f:
        f.write(mbr)

    # ── GPT Header (LBA 1) ──
    def make_gpt_header(my_lba, alt_lba, pe_start, pe_count, pe_size, disk_uuid):
        h = bytearray(SECTOR)
        h[0:8] = b"EFI PART"
        struct.pack_into("<I", h, 8, 0x00010000)
        struct.pack_into("<I", h, 12, HEADER_SIZE)
        struct.pack_into("<I", h, 16, 0)  # crc placeholder
        struct.pack_into("<Q", h, 24, my_lba)
        struct.pack_into("<Q", h, 32, alt_lba)
        struct.pack_into("<Q", h, 40, pe_start)
        struct.pack_into("<Q", h, 48, pe_count)
        struct.pack_into("<I", h, 56, pe_size)
        # First usable LBA
        usable_start = pe_start + (pe_count * pe_size + SECTOR - 1) // SECTOR
        struct.pack_into("<Q", h, 56, usable_start)
        # Last usable LBA
        usable_end = alt_lba - 1
        struct.pack_into("<Q", h, 64, usable_end)
        h[72:88] = disk_uuid
        # Partition entry array info
        struct.pack_into("<Q", h, 80, pe_count)
        struct.pack_into("<I", h, 84, pe_size)
        # CRC32 of header
        tmp = bytearray(h)
        struct.pack_into("<I", tmp, 16, 0)
        struct.pack_into("<I", h, 16, bin_crc32(tmp[:HEADER_SIZE]))
        return h

    disk_uuid = os.urandom(16)
    gpt = make_gpt_header(1, total_lba - 1, GPT_ENTRIES_LBA, GPT_ENTRIES_COUNT, GPT_ENTRY_SIZE, disk_uuid)

    with open(OUTPUT, "r+b") as f:
        f.seek(SECTOR)
        f.write(gpt)

    # ── Partition entries (LBA 2-33) ──
    pe = bytearray(GPT_ENTRIES_COUNT * GPT_ENTRY_SIZE)
    for i in range(GPT_ENTRIES_COUNT):
        if i == 0:
            # ESP partition
            pe[i*128:i*128+16] = bytes.fromhex("28732AC11FF8D211BA4B00A0C93EC93B")
            pe[i*128+16:i*128+32] = os.urandom(16)
            struct.pack_into("<Q", pe, i*128+32, esp_start)
            struct.pack_into("<Q", pe, i*128+40, esp_lba)
            pe[i*128+48] = 0  # attributes
            name = "CRONOS ESP\0".encode("utf-16-le")
            pe[i*128+56:i*128+56+len(name)] = name

    with open(OUTPUT, "r+b") as f:
        f.seek(GPT_ENTRIES_LBA * SECTOR)
        f.write(pe)

    # ── Backup GPT ──
    backup_gpt = make_gpt_header(total_lba - 1, 1, total_lba - 33, GPT_ENTRIES_COUNT, GPT_ENTRY_SIZE, disk_uuid)
    with open(OUTPUT, "r+b") as f:
        f.seek((total_lba - 1) * SECTOR)
        f.write(backup_gpt)
        f.seek((total_lba - 33) * SECTOR)
        f.write(pe)

    # ── Create FAT32 filesystem using FATtools ──
    print("[BUILD] Creating FAT32 filesystem...")
    from FATtools.mkfat import fat_mkfs
    from io import BytesIO
    
    # Read all files into memory
    with open(LIMINE_CFG, "rb") as fc:
        cfg_data = fc.read()
    with open(KERNEL, "rb") as fk:
        kernel_data = fk.read()
    with open(BOOTX64, "rb") as fb:
        bootx64_data = fb.read()
    
    # Create a BytesIO buffer for the FAT32 filesystem
    fat_buffer = BytesIO()
    fat_buffer.seek(esp_bytes - 1)
    fat_buffer.write(b'\x00')
    fat_buffer.seek(0)
    
    # Format as FAT32 using FATtools
    result = fat_mkfs(fat_buffer, esp_bytes)
    print(f"  FAT32 format result: {result}")
    
    # Write the FAT32 filesystem directly to the disk image at ESP partition offset
    fat_buffer.seek(0)
    fat_data = fat_buffer.read()
    
    with open(OUTPUT, "r+b") as f:
        f.seek(esp_offset)
        f.write(fat_data)
    
    # For now, append files directly to the partition
    # This is a simplified approach - for production use proper FAT file operations
    print("[BUILD] Writing files to partition (simplified approach)...")
    file_offset = esp_offset + 16384  # Start after boot sector area
    
    with open(OUTPUT, "r+b") as f:
        f.seek(file_offset)
        f.write(kernel_data)
        file_offset += len(kernel_data)
        f.seek(file_offset)
        f.write(cfg_data)
        file_offset += len(cfg_data)
        f.seek(file_offset)
        f.write(bootx64_data)

    # ── Install Limine bootloader ──
    print("[BUILD] Installing Limine bootloader...")
    r = subprocess.run([LIMINE_EXE, "bios-install", OUTPUT], capture_output=True, text=True)
    if r.returncode != 0:
        print(f"[WARN] limine install failed: {r.stderr.strip() or r.stdout.strip()}")
    else:
        print(f"  limine install OK: {r.stdout.strip()}")

    actual = os.path.getsize(OUTPUT)
    print(f"[BUILD] Done! Image: {OUTPUT} ({actual//1024//1024} MB)")

def bin_crc32(data):
    import binascii
    return binascii.crc32(bytes(data)) & 0xFFFFFFFF

if __name__ == "__main__":
    main()
