import struct, io
from FATtools.mkfat import fat_mkfs

size = 16 * 1024 * 1024
buf = io.BytesIO()
buf.seek(size - 1)
buf.write(b'\x00')
buf.seek(0)
r = fat_mkfs(buf, size)
buf.seek(0)
bpb = buf.read(512)

bps = struct.unpack_from('<H', bpb, 11)[0]
spc = bpb[13]
rsvd = struct.unpack_from('<H', bpb, 14)[0]
fats = bpb[16]
root_entries = struct.unpack_from('<H', bpb, 17)[0]
total16 = struct.unpack_from('<H', bpb, 19)[0]
media = bpb[21]
fat16sz = struct.unpack_from('<H', bpb, 22)[0]
sec_per_trk = struct.unpack_from('<H', bpb, 24)[0]
heads = struct.unpack_from('<H', bpb, 26)[0]
hidden = struct.unpack_from('<I', bpb, 28)[0]
total32 = struct.unpack_from('<I', bpb, 32)[0]

drive_num = bpb[36] if total32 == 0 else bpb[64]
boot_sig = bpb[38] if total32 == 0 else bpb[66]
vol_id = struct.unpack_from('<I', bpb, 39 if total32 == 0 else 67)[0]
vol_label = bpb[43:54 if total32 == 0 else 75].rstrip(b' ')
fs_type = bpb[54:61 if total32 == 0 else 82].rstrip(b' ')
fat32_sz = struct.unpack_from('<I', bpb, 36)[0] if total32 != 0 else 0

print(f"bps={bps} spc={spc} rsvd={rsvd} fats={fats}")
print(f"root_entries={root_entries} total16={total16} media=0x{media:02x}")
print(f"fat16sz={fat16sz} total32={total32} fat32_sz={fat32_sz}")
print(f"sec_per_trk={sec_per_trk} heads={heads} hidden={hidden}")
print(f"drive_num={drive_num} boot_sig={boot_sig}")
print(f"vol_id=0x{vol_id:08x} vol_label={vol_label!r} fs_type={fs_type!r}")

# Detect filesystem type
if total32 != 0 or fat32_sz != 0 or rsvd > 1:
    fstype = "FAT32"
elif total16 < 4085:
    fstype = "FAT12"
else:
    fstype = "FAT16"
print(f"\nDetected: {fstype}")

# Total clusters
root_sectors = (root_entries * 32 + bps - 1) // bps
data_sectors = total16 - (rsvd + fats * fat16sz + root_sectors)
total_clusters = data_sectors // spc
print(f"root_sectors={root_sectors} data_sectors={data_sectors} clusters={total_clusters}")
