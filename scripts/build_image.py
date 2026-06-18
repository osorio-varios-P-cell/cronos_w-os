#!/usr/bin/env python3
"""Build a bootable GPT disk image for CRONOS OS with Limine (BIOS + UEFI)."""
import os, struct, subprocess

WORK = r"F:\cronos_w-os-github"
KERNEL = os.path.join(WORK, "target", "x86_64-unknown-none", "release", "cronos_w_os")
LIMINE_CFG = os.path.join(WORK, "limine.cfg")
LIMINE_DIR = os.path.join(WORK, "limine-tools", "limine-binary")
LIMINE_EXE = os.path.join(LIMINE_DIR, "limine-tool-windows-x86", "limine.exe")
BOOTX64 = os.path.join(LIMINE_DIR, "BOOTX64.EFI")
LIMINE_SYS = os.path.join(LIMINE_DIR, "limine-bios.sys")
OUTPUT = os.path.join(WORK, "cronos_w-os.img")

SECTOR = 512
GPT_ENTRIES_LBA = 2
GPT_ENTRY_SIZE = 128
GPT_ENTRIES_COUNT = 128
PE_TOTAL_LBAS = (GPT_ENTRIES_COUNT * GPT_ENTRY_SIZE + SECTOR - 1) // SECTOR  # 32

# GPT Partition type GUIDs - mixed-endian (RFC 4122)
# EFI System Partition: C12A7328-F81F-11D2-BA4B-00A0C93EC93B
_ESP_GUID = (struct.pack('<IHH', 0xC12A7328, 0xF81F, 0x11D2) + 
             struct.pack('>H', 0xBA4B) + 
             bytes.fromhex("00A0C93EC93B"))
ESP_TYPE_GUID = _ESP_GUID

# BIOS boot partition: 21686148-6449-6E6F-744E-656564454649
_BIOS_BOOT_GUID = (struct.pack('<IHH', 0x21686148, 0x6449, 0x6E6F) + 
                   struct.pack('>H', 0x744E) + 
                   bytes.fromhex("656564454649"))
BIOS_BOOT_TYPE_GUID = _BIOS_BOOT_GUID

def crc32(data):
    import binascii
    return binascii.crc32(bytes(data)) & 0xFFFFFFFF

def make_gpt_header(my_lba, alt_lba, pe_lba, pe_count, pe_size, disk_uuid,
                    first_usable, last_usable, pe_crc=0):
    h = bytearray(SECTOR)
    struct.pack_into("<8sI", h, 0, b"EFI PART", 0x00010000)
    struct.pack_into("<I", h, 12, 92)
    struct.pack_into("<I", h, 16, 0)
    struct.pack_into("<I", h, 20, 0)
    struct.pack_into("<QQ", h, 24, my_lba, alt_lba)
    struct.pack_into("<QQ", h, 40, first_usable, last_usable)
    h[56:72] = disk_uuid
    struct.pack_into("<Q", h, 72, pe_lba)
    struct.pack_into("<I", h, 80, pe_count)
    struct.pack_into("<I", h, 84, pe_size)
    struct.pack_into("<I", h, 88, pe_crc)
    struct.pack_into("<I", h, 16, crc32(h[:92]))
    return h

def make_partition_entry(type_guid, unique_guid, first_lba, last_lba, attrs, name):
    """Create a 128-byte GPT partition entry."""
    pe = bytearray(GPT_ENTRY_SIZE)
    pe[0:16] = type_guid
    pe[16:32] = unique_guid
    struct.pack_into("<QQ", pe, 32, first_lba, last_lba)
    struct.pack_into("<Q", pe, 48, attrs)
    name_utf16 = name.encode("utf-16-le")
    pe[56:56+len(name_utf16)] = name_utf16
    return pe

def fix_fat32_bpb(boot_sector, partition_lba, partition_sectors):
    """Fix the FAT32 BPB fields that fat_mkfs gets wrong."""
    bs = bytearray(boot_sector)
    
    bps = 512
    spc = 1  # 512-byte clusters for proper FAT32 cluster count
    rsvd = 32  # reserved sectors (includes backup boot sector)
    fats = 2
    root_entries = 0  # FAT32
    media = 0xF8
    fat16sz = 0
    hidden = partition_lba
    
    total_sectors = partition_sectors
    # For FAT32 with spc=1: need fat32_sz = 1024 sectors per FAT
    fat32_sz = 1024
    
    root_clus = 2  # First data cluster for FAT32 root directory
    fs_info = 1
    backup_boot = 6
    
    drive_num = 0x80
    boot_sig = 0x29
    vol_id = 0x12345678
    vol_label = b"CRONOS_ESP "
    fs_type = b"FAT32   "
    
    # Write standard BPB
    struct.pack_into("<H", bs, 11, bps)           # Bytes per sector
    bs[13] = spc                                   # Sectors per cluster
    struct.pack_into("<H", bs, 14, rsvd)          # Reserved sectors
    bs[16] = fats                                  # Number of FATs
    struct.pack_into("<H", bs, 17, root_entries)  # Root entries (0 for FAT32)
    struct.pack_into("<H", bs, 19, 0)             # Total sectors 16-bit (0 for FAT32)
    bs[21] = media                                 # Media type
    struct.pack_into("<H", bs, 22, fat16sz)       # FAT size 16-bit (0 for FAT32)
    struct.pack_into("<H", bs, 24, 63)            # Sectors per track
    struct.pack_into("<H", bs, 26, 255)           # Heads
    struct.pack_into("<I", bs, 28, hidden)        # Hidden sectors
    struct.pack_into("<I", bs, 32, total_sectors) # Total sectors 32-bit
    
    # FAT32 extended BPB (offset 36)
    struct.pack_into("<I", bs, 36, fat32_sz)      # FAT size 32-bit
    struct.pack_into("<H", bs, 40, 0)             # Ext flags (mirror enabled)
    struct.pack_into("<H", bs, 42, 0)             # FS version
    struct.pack_into("<I", bs, 44, root_clus)     # Root cluster
    struct.pack_into("<H", bs, 48, fs_info)       # FSInfo sector
    struct.pack_into("<H", bs, 50, backup_boot)   # Backup boot sector
    bs[52:64] = b"\x00" * 12                      # Reserved
    bs[64] = drive_num                            # Drive number
    bs[65] = 0                                    # Reserved
    bs[66] = boot_sig                             # Extended boot signature
    struct.pack_into("<I", bs, 67, vol_id)        # Volume ID
    bs[71:82] = vol_label                         # Volume label
    bs[82:90] = fs_type                           # FS type
    
    # Boot signature
    bs[510] = 0x55
    bs[511] = 0xAA
    
    return bs

def main():
    img_mb = 128
    img_size = img_mb * 1024 * 1024
    total_lba = img_size // SECTOR
    first_usable = 34
    
    # Partition layout:
    # LBA 0: Protective MBR
    # LBA 1: GPT Header
    # LBA 2-33: Partition entries (32 sectors)
    # LBA 34-65: BIOS boot partition (32 KiB = 64 sectors) - for Limine stage 2
    # LBA 66-66+ESP: ESP partition
    
    bios_boot_start = first_usable  # 34
    bios_boot_sectors = 64  # 32 KiB
    bios_boot_end = bios_boot_start + bios_boot_sectors - 1  # 97
    
    esp_start_lba = bios_boot_end + 1  # 98
    esp_sectors = 64 * 1024 * 1024 // SECTOR  # 64MB = 131072 sectors
    esp_end_lba = esp_start_lba + esp_sectors - 1
    esp_offset = esp_start_lba * SECTOR
    esp_bytes = esp_sectors * SECTOR
    
    # FAT32 parameters (must match fix_fat32_bpb)
    # For FAT32: need >= 65525 clusters
    # 64MB = 131072 sectors, with spc=1 -> ~131000 clusters (valid FAT32)
    rsvd = 32
    spc = 1
    fats = 2
    # fat32_sz = (total_sectors - rsvd) / (fats + 128*spc/512) approximation
    # For 131072 sectors, rsvd=32, fats=2, spc=1:
    # data_sectors = 131072 - 32 = 131040
    # clusters = 131040 / 1 = 131040
    # fat_entries = 131040 + 2 = 131042
    # fat_bytes = 131042 * 4 = 524168 bytes
    # fat_sectors = 524168 / 512 = 1024 sectors
    fat32_sz = 1024
    root_clus = 2
    
    last_usable = total_lba - 1 - PE_TOTAL_LBAS - 1
    
    print(f"[BUILD] Image: {img_mb}MB ({total_lba} sectors)")
    print(f"[BUILD] BIOS boot partition: LBA {bios_boot_start}-{bios_boot_end} ({bios_boot_sectors} sectors, 32 KiB)")
    print(f"[BUILD] ESP partition: LBA {esp_start_lba}-{esp_end_lba} ({esp_sectors} sectors, {esp_bytes//1024//1024} MB)")

    # ── Create GPT image ──
    disk_uuid = os.urandom(16)
    img = bytearray(img_size)

    # Protective MBR at LBA 0
    mbr = bytearray(SECTOR)
    mbr[0x1BE] = 0x00
    mbr[0x1BF:0x1C2] = b"\x00\x02\x00"
    mbr[0x1C2] = 0xEE
    mbr[0x1C3:0x1C6] = b"\xFF\xFF\xFF"
    struct.pack_into("<I", mbr, 0x1C6, 1)
    struct.pack_into("<I", mbr, 0x1CA, min(total_lba - 1, 0xFFFFFFFF))
    mbr[510:512] = b"\x55\xAA"
    img[0:SECTOR] = mbr

    # Partition entries (2 partitions)
    pe = bytearray(GPT_ENTRIES_COUNT * GPT_ENTRY_SIZE)
    
    # Partition 1: BIOS boot (Limine stage 2)
    # GPT attribute bit 2 (value 4) = legacy BIOS bootable
    pe[0:128] = make_partition_entry(
        BIOS_BOOT_TYPE_GUID,
        os.urandom(16),
        bios_boot_start,
        bios_boot_end,
        4,  # legacy BIOS bootable attribute
        "Limine BIOS"
    )
    
    # Partition 2: ESP (EFI System Partition)
    pe[128:256] = make_partition_entry(
        ESP_TYPE_GUID,
        os.urandom(16),
        esp_start_lba,
        esp_end_lba,
        0,
        "CRONOS ESP"
    )

    # Primary GPT header at LBA 1
    pe_crc = crc32(pe)
    gpt = make_gpt_header(1, total_lba - 1,
                          GPT_ENTRIES_LBA, GPT_ENTRIES_COUNT, GPT_ENTRY_SIZE,
                          disk_uuid, first_usable, last_usable, pe_crc)
    img[SECTOR:2*SECTOR] = gpt

    # Partition entries at LBA 2-33
    pe_offset = GPT_ENTRIES_LBA * SECTOR
    img[pe_offset:pe_offset + len(pe)] = pe

    # Backup partition entries
    bpe_lba = last_usable + 1
    bpe_offset = bpe_lba * SECTOR
    img[bpe_offset:bpe_offset + len(pe)] = pe

    # Backup GPT header at last LBA
    bgpt = make_gpt_header(total_lba - 1, 1,
                           bpe_lba, GPT_ENTRIES_COUNT, GPT_ENTRY_SIZE,
                           disk_uuid, first_usable, last_usable, pe_crc)
    bgpt_offset = (total_lba - 1) * SECTOR
    img[bgpt_offset:bgpt_offset + SECTOR] = bgpt

    # ── Build FAT32 filesystem in the ESP partition area ──
    print("[BUILD] Creating FAT32 filesystem...")
    from FATtools.mkfat import fat_mkfs
    from io import BytesIO

    fat = BytesIO()
    fat.seek(esp_bytes - 1)
    fat.write(b"\x00")
    fat.seek(0)
    r = fat_mkfs(fat, esp_bytes)
    print(f"  FATtools format result: {r}")

    # Write FAT data into image at ESP offset
    fat.seek(0)
    fat_data = fat.read()
    img[esp_offset:esp_offset + len(fat_data)] = fat_data

    # ── Fix BPB in the ESP boot sector ──
    print("[BUILD] Fixing FAT32 BPB...")
    fixed_bpb = fix_fat32_bpb(img[esp_offset:esp_offset+SECTOR], esp_start_lba, esp_sectors)
    img[esp_offset:esp_offset+SECTOR] = fixed_bpb

    print("[BUILD] Writing files to FAT partition...")
    from FATtools.FAT import FAT

    class FakePartition:
        def __init__(self, img_data, offset, size):
            self.img = img_data
            self.offset = offset
            self.size = size
            self.pos = 0
            self.mbr = None
            self.mode = 'r+b'
        def seek(self, pos, whence=0):
            if whence == 0:
                self.pos = pos
            elif whence == 1:
                self.pos += pos
            elif whence == 2:
                self.pos = self.size + pos
            return self.pos
        def read(self, n=-1):
            if n < 0 or self.pos + n > self.size:
                n = self.size - self.pos
            end = self.offset + self.pos + n
            data = self.img[self.offset + self.pos:end]
            self.pos += len(data)
            return data
        def write(self, data):
            n = len(data)
            end = self.offset + self.pos + n
            self.img[self.offset + self.pos:end] = data[:n]
            self.pos += n
            return n
        def tell(self):
            return self.pos
        def close(self):
            pass
        def flush(self):
            pass

    with open(KERNEL, "rb") as f:
        kdata = f.read()
    with open(LIMINE_CFG, "rb") as f:
        cdata = f.read()
    with open(BOOTX64, "rb") as f:
        bdata = f.read()
    with open(LIMINE_SYS, "rb") as f:
        limine_sys = f.read()

    part = FakePartition(img, esp_offset, esp_bytes)
    from FATtools.FAT import FATException
    try:
        from FATtools.Volume import openvolume
        vol = openvolume(part)
        print(f"  Volume type: {type(vol).__name__}")
        orig_open = vol.open
        def patched_open(name):
            try:
                return orig_open(name)
            except FATException:
                class FH: IsValid = False
                return FH()
        vol.open = patched_open
        # Create directory structure (mkdir supports nested paths)
        vol.mkdir("EFI")
        vol.mkdir("EFI/BOOT")
        
        # Create all files in root (Limine BIOS searches root, /boot, /limine, /boot/limine)
        h = vol.create("limine.conf")
        h.write(cdata)
        h.close()
        h = vol.create("CRONOS")
        h.write(kdata)
        h.close()
        h = vol.create("limine-bios.sys")
        h.write(limine_sys)
        h.close()
        
        # Try to create BOOTX64.EFI in EFI/BOOT using full path
        # Some FATtools versions support this
        try:
            h = vol.create("EFI\\BOOT\\BOOTX64.EFI")
            h.write(bdata)
            h.close()
        except:
            # Fallback: create in root
            h = vol.create("BOOTX64.EFI")
            h.write(bdata)
            h.close()
        print(f"  Files written OK (kernel={len(kdata)}, cfg={len(cdata)}, efi={len(bdata)}, limine_sys={len(limine_sys)})")
    except Exception as e:
        import traceback
        print(f"  Volume I/O failed: {e}")
        traceback.print_exc()
        print("  Using raw sector fallback...")
        off = esp_offset + 16384
        img[off:off+len(kdata)] = kdata
        off += len(kdata) + 512
        img[off:off+len(cdata)] = cdata
        off += len(cdata) + 512
        img[off:off+len(bdata)] = bdata

    # ── Mirror FAT 0 to FAT 1 and fix reserved entries (AFTER file writes) ──
    print("[BUILD] Fixing FAT tables (mirror + reserved)...")
    fat0_start = esp_offset + rsvd * SECTOR
    fat1_start = fat0_start + fat32_sz * SECTOR
    fat_size = fat32_sz * SECTOR
    
    # Copy FAT 0 to FAT 1
    img[fat1_start:fat1_start + fat_size] = img[fat0_start:fat0_start + fat_size]
    
    # Fix reserved cluster entries in both FATs
    # Cluster 0: media type (lower 8 bits) + 0xFFFFFF00
    # For FAT32 on hard disk: 0xFFFFFFF8
    # Cluster 1: 0xFFFFFFFF (end of chain)
    struct.pack_into("<I", img, fat0_start + 0, 0xFFFFFFF8)
    struct.pack_into("<I", img, fat0_start + 4, 0xFFFFFFFF)
    struct.pack_into("<I", img, fat1_start + 0, 0xFFFFFFF8)
    struct.pack_into("<I", img, fat1_start + 4, 0xFFFFFFFF)

    # ── Write image to disk ──
    print(f"[BUILD] Writing image ({len(img)} bytes)...")
    with open(OUTPUT, "wb") as f:
        f.write(img)

    # ── Install Limine bootloader (BIOS/GPT) ──
    # Partition 1 is the BIOS boot partition (1-based index)
    print("[BUILD] Installing Limine bootloader (BIOS/GPT)...")
    try:
        r = subprocess.run([LIMINE_EXE, "bios-install", OUTPUT, "1"],
                           capture_output=True, text=True, timeout=15)
        if r.returncode != 0:
            print(f"[WARN] limine install: {r.stderr.strip() or r.stdout.strip()}")
        else:
            print(f"  OK: {r.stdout.strip()}")
    except subprocess.TimeoutExpired:
        print("[WARN] limine timed out (may still have written data)")

    actual = os.path.getsize(OUTPUT)
    print(f"[BUILD] Done! {OUTPUT} ({actual//1024//1024} MB)")

if __name__ == "__main__":
    main()