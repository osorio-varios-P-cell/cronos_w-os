import struct, io
from FATtools.mkfat import fat_mkfs

for size in [16*1024*1024, 32*1024*1024, 64*1024*1024, 128*1024*1024]:
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
    fat16sz = struct.unpack_from('<H', bpb, 22)[0]
    total32 = struct.unpack_from('<I', bpb, 32)[0]
    fat32_sz = struct.unpack_from('<I', bpb, 36)[0]
    root_clus = struct.unpack_from('<I', bpb, 44)[0]
    fs_type = bpb[54:61 if total32 == 0 else 82].rstrip(b' ')
  
    if total32 != 0 or rsvd > 1:
        fstype = "FAT32"
    elif total16 < 4085:
        fstype = "FAT12"
    else:
        fstype = "FAT16"
  
    root_sectors = (root_entries * 32 + bps - 1) // bps
    fs = fat32_sz if fat32_sz else fat16sz
    ds = total32 if total32 else total16
    data_sectors = ds - (rsvd + fats * fs + root_sectors)
    clusters = data_sectors // spc
  
    print(f"--- Size={size//1024//1024}MB (r={r}) ---")
    print(f"  fstype={fstype} bps={bps} spc={spc}({spc*512//1024}KB)")
    print(f"  rsvd={rsvd} fats={fats} root_entries={root_entries}")
    print(f"  total16={total16} total32={total32}")
    print(f"  fat16sz={fat16sz} fat32_sz={fat32_sz}")
    print(f"  root_clus={root_clus} fs_type_str={fs_type!r}")
    print(f"  data_sectors={data_sectors} clusters={clusters}")
    print()
