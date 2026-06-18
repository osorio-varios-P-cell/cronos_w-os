import struct, os
from io import BytesIO
from FATtools.mkfat import fat_mkfs
from FATtools.FAT import FATException
from FATtools.Volume import openvolume

esp_bytes = 64 * 1024 * 1024

# Create FAT image in memory
fat = BytesIO()
fat.seek(esp_bytes - 1)
fat.write(b"\x00")
fat.seek(0)
r = fat_mkfs(fat, esp_bytes)
print(f"fat_mkfs result: {r}")

fat.seek(0)
bpb = fat.read(512)

def fix_fat32_bpb(boot_sector, partition_lba, partition_sectors):
    bs = bytearray(boot_sector)
    bps = 512
    spc = 64
    rsvd = 32
    fats = 2
    root_entries = 0
    media = 0xF8
    fat16sz = 0
    hidden = partition_lba
    total_sectors = partition_sectors
    fat32_sz = 16
    root_clus = 2
    fs_info = 1
    backup_boot = 6
    drive_num = 0x80
    boot_sig = 0x29
    vol_id = 0x12345678
    vol_label = b"CRONOS_ESP "
    fs_type = b"FAT32   "
    
    struct.pack_into("<H", bs, 11, bps)
    bs[13] = spc
    struct.pack_into("<H", bs, 14, rsvd)
    bs[16] = fats
    struct.pack_into("<H", bs, 17, root_entries)
    struct.pack_into("<H", bs, 19, 0)
    bs[21] = media
    struct.pack_into("<H", bs, 22, fat16sz)
    struct.pack_into("<H", bs, 24, 63)
    struct.pack_into("<H", bs, 26, 255)
    struct.pack_into("<I", bs, 28, hidden)
    struct.pack_into("<I", bs, 32, total_sectors)
    struct.pack_into("<I", bs, 36, fat32_sz)
    struct.pack_into("<H", bs, 40, 0)
    struct.pack_into("<H", bs, 42, 0)
    struct.pack_into("<I", bs, 44, root_clus)
    struct.pack_into("<H", bs, 48, fs_info)
    struct.pack_into("<H", bs, 50, backup_boot)
    bs[52:64] = b"\x00" * 12
    bs[64] = drive_num
    bs[65] = 0
    bs[66] = boot_sig
    struct.pack_into("<I", bs, 67, vol_id)
    bs[71:82] = vol_label
    bs[82:90] = fs_type
    bs[510] = 0x55
    bs[511] = 0xAA
    return bs

fixed_bpb = fix_fat32_bpb(bpb, 98, 131072)
fat.seek(0)
fat.write(fixed_bpb)
fat.seek(0)

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

img_data = bytearray(fat.getvalue())
part = FakePartition(img_data, 0, esp_bytes)
vol = openvolume(part)
print(f"Volume type: {type(vol).__name__}")

# Check what methods exist
print("Methods:", [m for m in dir(vol) if not m.startswith('_')])

# Try listdir without args
try:
    print("listdir():", vol.listdir())
except Exception as e:
    print(f"listdir() failed: {e}")

# Try create file in root
try:
    h = vol.create("test.txt")
    print(f"create test.txt: {h}")
    if h and hasattr(h, 'IsValid') and h.IsValid != False:
        h.write(b"hello")
        h.close()
        print("  wrote test.txt")
except Exception as e:
    print(f"create failed: {e}")
    import traceback
    traceback.print_exc()

# Check if EFI dir was created
try:
    print("listdir after create:", vol.listdir())
except Exception as e:
    print(f"listdir failed: {e}")

# Try creating subdir
try:
    vol.mkdir("EFI")
    print("mkdir EFI: success")
    print("listdir after mkdir:", vol.listdir())
except Exception as e:
    print(f"mkdir failed: {e}")

# Try open EFI
try:
    efi = vol.open("EFI")
    print(f"open EFI: {efi}")
    if efi:
        print(f"EFI IsValid: {efi.IsValid}")
except Exception as e:
    print(f"open EFI failed: {e}")

# Try listdir of EFI if it exists
try:
    for name in vol.listdir("EFI"):
        print(f"  EFI/{name}")
except Exception as e:
    print(f"listdir EFI failed: {e}")

# Try using create for directories too
try:
    vol.create("BOOT")
    print("create BOOT: success")
    print("listdir:", vol.listdir())
except Exception as e:
    print(f"create BOOT failed: {e}")
