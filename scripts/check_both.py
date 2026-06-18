import struct

with open('cronos_w-os.img', 'rb') as f:
    # Read GPT header at LBA 1
    f.seek(512)
    gpt = f.read(92)
    pe_lba = struct.unpack_from('<Q', gpt, 72)[0]
    # Read partition entries
    f.seek(pe_lba * 512)
    pe_data = f.read(128 * 2)  # Read first 2 entries
    
    for i in range(2):
        pe = pe_data[i*128:(i+1)*128]
        type_guid = pe[0:16]
        start_lba = struct.unpack_from('<Q', pe, 32)[0]
        end_lba = struct.unpack_from('<Q', pe, 40)[0]
        name = pe[56:128].decode('utf-16-le', errors='ignore').rstrip('\x00')
        print(f"Partition {i+1}: type={type_guid.hex()}, LBA {start_lba}-{end_lba}, name={name}")
        
        # Read BPB from this partition
        f.seek(start_lba * 512)
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
        root_clus = struct.unpack_from('<I', bpb, 44)[0]
        hidden = struct.unpack_from('<I', bpb, 28)[0]
        vol_label = bpb[71:82].rstrip(b' ')
        fs_type = bpb[82:90].rstrip(b' ')
        
        print(f"  bps={bps} spc={spc} rsvd={rsvd} fats={fats}")
        print(f"  root_entries={root_entries} total16={total16} total32={total32}")
        print(f"  fat16sz={fat16sz} fat32_sz={fat32_sz} root_clus={root_clus}")
        print(f"  hidden={hidden} vol_label={vol_label!r} fs_type={fs_type!r}")
        
        if total32 != 0 or fat32_sz != 0 or rsvd > 1:
            fstype = "FAT32"
        elif total16 < 4085:
            fstype = "FAT12"
        else:
            fstype = "FAT16"
        print(f"  Detected: {fstype}")
        print()
