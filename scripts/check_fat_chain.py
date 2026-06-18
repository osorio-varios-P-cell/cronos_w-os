import struct

with open('cronos_w-os.img', 'rb') as f:
    esp_start = 98
    f.seek(esp_start * 512)
    bpb = f.read(512)
    
    bps = struct.unpack_from('<H', bpb, 11)[0]
    spc = bpb[13]
    rsvd = struct.unpack_from('<H', bpb, 14)[0]
    fats = bpb[16]
    fat32_sz = struct.unpack_from('<I', bpb, 36)[0]
    root_clus = struct.unpack_from('<I', bpb, 44)[0]
    
    fat_start = esp_start + rsvd
    fat_offset = fat_start * 512
    
    # Read FAT table
    fat_size = fat32_sz * 512
    f.seek(fat_offset)
    fat_data = f.read(fat_size)
    
    # Check cluster 135 (limine-bios.sys)
    entry_offset = 135 * 4
    entry = struct.unpack_from('<I', fat_data, entry_offset)[0]
    print(f"FAT entry for cluster 135: 0x{entry:08X} ({entry})")
    
    # Follow the chain
    cluster = 135
    count = 0
    while cluster < 0x0FFFFFF8 and count < 20:
        entry_offset = cluster * 4
        entry = struct.unpack_from('<I', fat_data, entry_offset)[0]
        print(f"  Cluster {cluster} -> {entry} (0x{entry:08X})")
        cluster = entry
        count += 1
    
    # Also check cluster 142 (BOOTX64.EFI)
    cluster = 142
    count = 0
    print(f"\nFAT entry for cluster 142:")
    while cluster < 0x0FFFFFF8 and count < 20:
        entry_offset = cluster * 4
        entry = struct.unpack_from('<I', fat_data, entry_offset)[0]
        print(f"  Cluster {cluster} -> {entry} (0x{entry:08X})")
        cluster = entry
        count += 1
    
    # Check cluster 4 (CRONOS OS)
    cluster = 4
    count = 0
    print(f"\nFAT entry for cluster 4:")
    while cluster < 0x0FFFFFF8 and count < 20:
        entry_offset = cluster * 4
        entry = struct.unpack_from('<I', fat_data, entry_offset)[0]
        print(f"  Cluster {cluster} -> {entry} (0x{entry:08X})")
        cluster = entry
        count += 1
