import os
import struct
import subprocess
import shutil
from FATtools.Volume import openvolume, vopen
from FATtools.mkfat import fat_mkfs

WORK = "."
KERNEL = "target/x86_64-unknown-none/release/cronos_w_os"
LIMINE_CONF = "limine.conf"
LIMINE_DIR = "limine-tools/limine-binary"
LIMINE_EXE = f"{LIMINE_DIR}/limine-tool-windows-x86/limine.exe"
BOOTX64 = f"{LIMINE_DIR}/BOOTX64.EFI" # UEFI boot file
OUTPUT = "cronos_w-os.img"

SECTOR = 512

def bin_crc32(data):
    import binascii
    return binascii.crc32(bytes(data)) & 0xFFFFFFFF

def main():
    img_mb = 512
    img_size = img_mb * 1024 * 1024
    total_lba = img_size // SECTOR
    esp_start = 2048 # Align at 1MB for maximum safety and compatibility
    esp_lba = (img_mb - 2) * 1024 * 1024 // SECTOR # Leave some space at the end

    esp_offset = esp_start * SECTOR
    esp_bytes = esp_lba * SECTOR

    print(f"[BUILD] Image: {img_mb}MB, ESP: LBA {esp_start}-{esp_start+esp_lba-1} ({esp_bytes//1024//1024}MB)")

    # 1. Create empty image
    with open(OUTPUT, "wb") as f:
        f.truncate(img_size)

    # 2. Write Protective MBR (LBA 0)
    mbr = bytearray(SECTOR)
    mbr[0x1BE] = 0x00 # Boot indicator
    mbr[0x1BF:0x1C2] = b"\x00\x02\x00" # CHS start
    mbr[0x1C2] = 0xEE # GPT Protective Partition type
    mbr[0x1C3:0x1C6] = b"\xFF\xFF\xFF" # CHS end
    struct.pack_into("<I", mbr, 0x1C6, 1) # Start LBA
    struct.pack_into("<I", mbr, 0x1CA, min(total_lba - 1, 0xFFFFFFFF)) # Size in sectors
    mbr[0x1FE:0x200] = b"\x55\xAA"
    with open(OUTPUT, "r+b") as f:
        f.write(mbr)

    # 3. Create GPT partition table (LBA 1 is header, 2-33 are entries)
    # Disk UUID and Partition UUID
    disk_uuid = os.urandom(16)
    part_uuid = os.urandom(16)

    # We will build partition entries
    pe = bytearray(128 * 128)
    # Basic Data Partition GUID: EBD0A0A2-B9E5-4433-87C0-68B6B72699C7
    pe[0:16] = bytes.fromhex("A2A0D0EBE5B9334487C068B6B72699C7")
    pe[16:32] = part_uuid
    struct.pack_into("<Q", pe, 32, esp_start) # Start LBA
    struct.pack_into("<Q", pe, 40, esp_start + esp_lba - 1) # End LBA (inclusive!)
    pe[48] = 0 # Attributes
    name = "CRONOS ESP\0".encode("utf-16-le")
    pe[56:56+len(name)] = name

    # Write partition entries (LBA 2)
    with open(OUTPUT, "r+b") as f:
        f.seek(2 * SECTOR)
        f.write(pe)

    # GPT Header LBA 1
    header = bytearray(SECTOR)
    header[0:8] = b"EFI PART"
    struct.pack_into("<I", header, 8, 0x00010000) # Revision 1.0
    struct.pack_into("<I", header, 12, 92) # Header size
    struct.pack_into("<I", header, 16, 0) # CRC placeholder
    struct.pack_into("<Q", header, 24, 1) # My LBA
    struct.pack_into("<Q", header, 32, total_lba - 1) # Alternate LBA
    struct.pack_into("<Q", header, 40, 34) # First usable LBA
    struct.pack_into("<Q", header, 48, total_lba - 34) # Last usable LBA
    header[56:72] = disk_uuid
    struct.pack_into("<Q", header, 72, 2) # Partition entries starting LBA
    struct.pack_into("<I", header, 80, 128) # Number of partition entries
    struct.pack_into("<I", header, 84, 128) # Size of partition entry
    # CRC of partition entries
    pe_crc = bin_crc32(pe)
    struct.pack_into("<I", header, 88, pe_crc)
    # Header CRC
    header_crc = bin_crc32(header[:92])
    struct.pack_into("<I", header, 16, header_crc)

    with open(OUTPUT, "r+b") as f:
        f.seek(1 * SECTOR)
        f.write(header)

    # 4. Backup GPT at the end of the disk
    # Backup partition entries array at LBA total_lba - 33
    with open(OUTPUT, "r+b") as f:
        f.seek((total_lba - 33) * SECTOR)
        f.write(pe)

    # Backup header at LBA total_lba - 1
    bheader = bytearray(header)
    struct.pack_into("<Q", bheader, 24, total_lba - 1) # My LBA
    struct.pack_into("<Q", bheader, 32, 1) # Alternate LBA
    struct.pack_into("<Q", bheader, 72, total_lba - 33) # Partition entries starting LBA
    struct.pack_into("<I", bheader, 16, 0) # CRC placeholder
    bheader_crc = bin_crc32(bheader[:92])
    struct.pack_into("<I", bheader, 16, bheader_crc)

    with open(OUTPUT, "r+b") as f:
        f.seek((total_lba - 1) * SECTOR)
        f.write(bheader)

    # 5. Format partition as FAT32
    print("[BUILD] Formatting partition as FAT32...")
    import io
    fat_buf = io.BytesIO(b'\x00' * esp_bytes)
    fat_mkfs(fat_buf, esp_bytes, params={'fat_bits': 32, 'wanted_cluster': 512})

    # Write formatted FAT32 filesystem to the partition offset
    fat_buf.seek(0)
    partition_data = bytearray(fat_buf.read())
    # Patch Hidden Sectors in FAT32 BPB (offset 0x1C) to match esp_start (2048)
    struct.pack_into("<I", partition_data, 0x1C, esp_start)
    
    with open(OUTPUT, "r+b") as f:
        f.seek(esp_offset)
        f.write(partition_data)

    # 6. Copy files using FATtools Volume API
    print("[BUILD] Copying files into the FAT32 partition...")
    # Open the formatted partition directly from the disk image file using vopen
    vol = vopen(OUTPUT, mode="r+b", what="auto")
    
    # Copy kernel, limine.cfg, limine-bios.sys
    def copy_file_to_vol(vol_obj, src_path, dst_name):
        with open(src_path, "rb") as sf:
            file_data = sf.read()
        f_entry = vol_obj.create(dst_name)
        f_entry.write(file_data)
        f_entry.close()
        print(f"  Copied {src_path} -> /{dst_name}")

    copy_file_to_vol(vol, KERNEL, "CRONOS")
    copy_file_to_vol(vol, LIMINE_CONF, "LIMINE.CONF")
    
    # Create STARTUP.NSH for UEFI shell autoboot
    startup_entry = vol.create("STARTUP.NSH")
    startup_entry.write(b"fs0:\\efi\\boot\\bootx64.efi\r\n")
    startup_entry.close()
    print("  Created /STARTUP.NSH")
    
    limine_bios = f"{LIMINE_DIR}/limine-bios.sys"
    if os.path.exists(limine_bios):
        copy_file_to_vol(vol, limine_bios, "LIMINE.SYS")

    # Create EFI/BOOT directory structure
    efi_dir = vol.mkdir("EFI")
    boot_dir = efi_dir.mkdir("BOOT")
    
    # Copy BOOTX64.EFI to boot_dir
    with open(BOOTX64, "rb") as sf:
        boot_data = sf.read()
    f_entry = boot_dir.create("BOOTX64.EFI")
    f_entry.write(boot_data)
    f_entry.close()
    print(f"  Copied {BOOTX64} -> /EFI/BOOT/BOOTX64.EFI")
    
    print("  [DEBUG] Root directory contents:", vol.listdir())
    vol.close()

    # 6.5. Run limine bios-install (Disabled to prevent GPT corruption/shrinking for UEFI boot compatibility)
    print("[BUILD] Skipping Limine BIOS stages installation for UEFI boot compatibility...")
    # r = subprocess.run([LIMINE_EXE, "bios-install", OUTPUT], capture_output=True, text=True)
    # if r.returncode != 0:
    #     print(f"[WARN] limine bios-install output: {r.stderr.strip() or r.stdout.strip()}")
    # else:
    #     print(f"  limine bios-install OK: {r.stdout.strip()}")

    # 7. Patch Partition Type GUID to EFI System Partition (ESP) GUID in R+B mode
    print("[BUILD] Patching GPT partition GUID to ESP type...")
    esp_guid = bytes.fromhex("28732AC11FF8D211BA4B00A0C93EC93B")
    
    with open(OUTPUT, "r+b") as f:
        # Read Primary GPT Header (92 bytes size)
        f.seek(1 * SECTOR)
        header_data = bytearray(f.read(92))
        
        pe_lba = struct.unpack_from("<Q", header_data, 72)[0]
        num_pe = struct.unpack_from("<I", header_data, 80)[0]
        sz_pe = struct.unpack_from("<I", header_data, 84)[0]
        
        print(f"  Primary GPT: PE LBA = {pe_lba}, Num PE = {num_pe}, Size PE = {sz_pe}")
        
        # Read the partition entry array
        f.seek(pe_lba * SECTOR)
        pe_data = bytearray(f.read(num_pe * sz_pe))
        
        # Modify the type GUID of the first entry
        pe_data[0:16] = esp_guid
        
        # Write back the modified partition entries
        f.seek(pe_lba * SECTOR)
        f.write(pe_data)
        
        # Read Backup GPT Header
        f.seek((total_lba - 1) * SECTOR)
        bheader_data = bytearray(f.read(92))
        bpe_lba = struct.unpack_from("<Q", bheader_data, 72)[0]
        
        print(f"  Backup GPT: PE LBA = {bpe_lba}")
        
        # Write to backup partition entries
        f.seek(bpe_lba * SECTOR)
        f.write(pe_data)
        
        # Recalculate CRC of partition entries
        pe_crc_new = bin_crc32(pe_data)
        
        # Update Primary Header
        struct.pack_into("<I", header_data, 88, pe_crc_new)
        struct.pack_into("<I", header_data, 16, 0) # Clear CRC placeholder
        header_crc_new = bin_crc32(header_data)
        struct.pack_into("<I", header_data, 16, header_crc_new)
        
        f.seek(1 * SECTOR)
        f.write(header_data)
        
        # Update Backup Header
        struct.pack_into("<I", bheader_data, 88, pe_crc_new)
        struct.pack_into("<I", bheader_data, 16, 0) # Clear CRC placeholder
        bheader_crc_new = bin_crc32(bheader_data)
        struct.pack_into("<I", bheader_data, 16, bheader_crc_new)
        
        f.seek((total_lba - 1) * SECTOR)
        f.write(bheader_data)

    print("[BUILD] Disk image built successfully!")

if __name__ == "__main__":
    main()
