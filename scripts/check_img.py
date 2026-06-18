import struct

with open('cronos_w-os.img', 'rb') as f:
    # Read GPT header at LBA 1
    f.seek(512)
    gpt = f.read(92)
    # Get partition start
    pe_lba = struct.unpack_from('<Q', gpt, 72)[0]
    # Read first partition entry
    f.seek(pe_lba * 512)
    pe = f.read(128)
    pe_start = struct.unpack_from('<Q', pe, 32)[0]
    pe_end = struct.unpack_from('<Q', pe, 40)[0]
    print(f"Partition: LBA {pe_start} to {pe_end} ({pe_end - pe_start + 1} sectors = {(pe_end - pe_start + 1) * 512 // 1024 // 1024} MB)")

    # Read BPB from partition start
    f.seek(pe_start * 512)
    bpb = f.read(512)
    
    bps = struct.unpack_from('<H', bpb, 11)[0]
    spc = bpb[13]
    rsvd = struct.unpack_from('<H', bpb, 14)[0]
    fats = bpb[16]
    root_entries = struct.unpack_from('<H', bpb, 17)[0]
    total16 = struct.unpack_from('<H', bpb, 19)[0]
    media = bpb[21]
    fat16sz = struct.unpack_from('<H', bpb, 22)[0]
    total32 = struct.unpack_from('<I', bpb, 32)[0]
    fat32_sz = struct.unpack_from('<I', bpb, 36)[0]
    hidden = struct.unpack_from('<I', bpb, 28)[0]
    
    vol_label = bpb[43:54].rstrip(b' ')
    fs_type = bpb[54:61].rstrip(b' ')
    
    print(f"bps={bps} spc={spc} rsvd={rsvd} fats={fats}")
    print(f"root_entries={root_entries} total16={total16} media=0x{media:02x}")
    print(f"fat16sz={fat16sz} total32={total32} fat32_sz={fat32_sz}")
    print(f"hidden={hidden}")
    print(f"vol_label={vol_label!r} fs_type={fs_type!r}")
    
    # Detect filesystem type
    if total32 != 0 or fat32_sz != 0 or rsvd > 1:
        fstype = "FAT32"
    elif total16 < 4085:
        fstype = "FAT12"
    else:
        fstype = "FAT16"
    print(f"\nDetected: {fstype}")
    
    root_sectors = (root_entries * 32 + bps - 1) // bps
    data_sectors = (total32 if total32 else total16) - (rsvd + fats * (fat32_sz if fat32_sz else fat16sz) + root_sectors)
    total_clusters = data_sectors // spc
    print(f"root_sectors={root_sectors} data_sectors={data_sectors} clusters={total_clusters}")
