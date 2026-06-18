import struct, os
from io import BytesIO
from FATtools.mkfat import fat_mkfs
from FATtools.FAT import FATException
from FATtools.Volume import openvolume

esp_bytes = 64 * 1024 * 1024

fat = BytesIO()
fat.seek(esp_bytes - 1)
fat.write(b"\x00")
fat.seek(0)
r = fat_mkfs(fat, esp_bytes)

fat.seek(0)
bpb = fat.read(512)

def fix_fat32_bpb(boot_sector, partition_lba, partition_sectors):
    bs = bytearray(boot_sector)
    bps = 512; spc = 64; rsvd = 32; fats = 2; root_entries = 0; media = 0xF8; fat16sz = 0
    hidden = partition_lba; total_sectors = partition_sectors; fat32_sz = 16; root_clus = 2
    fs_info = 1; backup_boot = 6; drive_num = 0x80; boot_sig = 0x29; vol_id = 0x12345678
    vol_label = b"CRONOS_ESP "; fs_type = b"FAT32   "
    struct.pack_into("<H", bs, 11, bps); bs[13] = spc
    struct.pack_into("<H", bs, 14, rsvd); bs[16] = fats
    struct.pack_into("<H", bs, 17, root_entries); struct.pack_into("<H", bs, 19, 0)
    bs[21] = media; struct.pack_into("<H", bs, 22, fat16sz)
    struct.pack_into("<H", bs, 24, 63); struct.pack_into("<H", bs, 26, 255)
    struct.pack_into("<I", bs, 28, hidden); struct.pack_into("<I", bs, 32, total_sectors)
    struct.pack_into("<I", bs, 36, fat32_sz); struct.pack_into("<H", bs, 40, 0)
    struct.pack_into("<H", bs, 42, 0); struct.pack_into("<I", bs, 44, root_clus)
    struct.pack_into("<H", bs, 48, fs_info); struct.pack_into("<H", bs, 50, backup_boot)
    bs[52:64] = b"\x00" * 12; bs[64] = drive_num; bs[65] = 0; bs[66] = boot_sig
    struct.pack_into("<I", bs, 67, vol_id); bs[71:82] = vol_label; bs[82:90] = fs_type
    bs[510] = 0x55; bs[511] = 0xAA
    return bs

fixed_bpb = fix_fat32_bpb(bpb, 98, 131072)
fat.seek(0)
fat.write(fixed_bpb)
fat.seek(0)

class FakePartition:
    def __init__(self, img_data, offset, size):
        self.img = img_data; self.offset = offset; self.size = size; self.pos = 0
        self.mbr = None; self.mode = 'r+b'
    def seek(self, pos, whence=0):
        if whence == 0: self.pos = pos
        elif whence == 1: self.pos += pos
        elif whence == 2: self.pos = self.size + pos
        return self.pos
    def read(self, n=-1):
        if n < 0 or self.pos + n > self.size: n = self.size - self.pos
        end = self.offset + self.pos + n
        data = self.img[self.offset + self.pos:end]
        self.pos += len(data)
        return data
    def write(self, data):
        n = len(data); end = self.offset + self.pos + n
        self.img[self.offset + self.pos:end] = data[:n]; self.pos += n; return n
    def tell(self): return self.pos
    def close(self): pass
    def flush(self): pass

img_data = bytearray(fat.getvalue())
part = FakePartition(img_data, 0, esp_bytes)
vol = openvolume(part)

# Create directory structure
vol.mkdir("EFI")
vol.mkdir("EFI/BOOT")

# Create files in root
h = vol.create("limine.cfg")
if h and h.IsValid != False:
    h.write(b"test config")
    h.close()

h = vol.create("limine-bios.sys")
if h and h.IsValid != False:
    h.write(b"fake limine-bios.sys")
    h.close()

# Create kernel file
h = vol.create("CRONOS OS")
if h and h.IsValid != False:
    h.write(b"X" * 1000)
    h.close()

# Create file in EFI/BOOT
# How to create in subdir? Try open the dir first
efi_handle = vol.open("EFI")
print(f"EFI handle: {efi_handle}")
print(f"EFI handle type: {type(efi_handle)}")

# Check if we can use the handle to create
if efi_handle:
    try:
        boot_handle = efi_handle.create("BOOTX64.EFI")
        print(f"BOOTX64 handle: {boot_handle}")
        if boot_handle and boot_handle.IsValid != False:
            boot_handle.write(b"fake BOOTX64.EFI")
            boot_handle.close()
    except Exception as e:
        print(f"create in subdir failed: {e}")

# List all
print("Root:", vol.listdir())
print("EFI:", vol.listdir("EFI"))  # Wait, listdir takes 0 args?