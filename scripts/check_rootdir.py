import struct

with open('cronos_w-os.img', 'rb') as f:
    # ESP partition starts at LBA 98
    esp_start = 98
    f.seek(esp_start * 512)
    bpb = f.read(512)
    
    bps = struct.unpack_from('<H', bpb, 11)[0]
    spc = bpb[13]
    rsvd = struct.unpack_from('<H', bpb, 14)[0]
    fats = bpb[16]
    fat32_sz = struct.unpack_from('<I', bpb, 36)[0]
    root_clus = struct.unpack_from('<I', bpb, 44)[0]
    
    print(f"bps={bps} spc={spc} rsvd={rsvd} fats={fats}")
    print(f"fat32_sz={fat32_sz} root_clus={root_clus}")
    
    # Calculate data start
    fat_start = esp_start + rsvd
    data_start = fat_start + fats * fat32_sz
    
    # For FAT32, cluster 2 starts at data_start
    cluster_size = spc * bps
    
    # Root directory is at cluster 2
    root_dir_lba = data_start + (root_clus - 2) * spc
    root_dir_offset = root_dir_lba * 512
    
    print(f"Root dir at LBA {root_dir_lba}, offset {hex(root_dir_offset)}")
    
    # Read root directory (several clusters)
    f.seek(root_dir_offset)
    root_data = f.read(4 * cluster_size)  # Read 4 clusters worth
    
    print(f"Read {len(root_data)} bytes from root dir")
    
    for i in range(0, len(root_data), 32):
        entry = root_data[i:i+32]
        if len(entry) < 32:
            break
        if entry[0] == 0:
            print(f"  {i//32:3d}: [END]")
            break
        if entry[0] == 0xE5:
            continue
        attr = entry[11]
        if attr == 0x0F:
            seq = entry[0] & 0x3F
            name_bytes = entry[1:11] + entry[14:26] + entry[28:32]
            try:
                name = name_bytes.decode('utf-16-le').rstrip('\uffff').rstrip('\x00')
            except:
                name = repr(name_bytes)
            print(f"  {i//32:3d}: VFAT seq={seq:2d} \"{name}\"")
        else:
            name = entry[0:8].rstrip(b' ').decode('ascii', errors='replace')
            ext = entry[8:11].rstrip(b' ').decode('ascii', errors='replace')
            full = (name + '.' + ext) if ext else name
            cluster = struct.unpack_from('<H', entry, 26)[0] | (struct.unpack_from('<H', entry, 20)[0] << 16)
            size = struct.unpack_from('<I', entry, 28)[0]
            is_dir = bool(attr & 0x10)
            print(f"  {i//32:3d}: SFN=\"{full}\" cluster={cluster} size={size} dir={is_dir}")
