import struct

with open('cronos_w-os.img', 'rb') as f:
    f.seek(512)
    gpt = f.read(512)
    pe_lba = struct.unpack('<Q', gpt[72:80])[0]
    print(f'Partition entries at LBA: {pe_lba}')

    f.seek(pe_lba * 512)
    pe = f.read(128)
    start_lba = struct.unpack('<Q', pe[32:40])[0]
    num_lba = struct.unpack('<Q', pe[40:48])[0]
    esp_off = start_lba * 512
    print(f'ESP partition: LBA {start_lba}, size {num_lba} LBAs')

    f.seek(esp_off)
    bpb = f.read(512)

    bytes_per_sec = struct.unpack_from('<H', bpb, 11)[0]
    sec_per_clus = bpb[13]
    rsvd_sec_cnt = struct.unpack_from('<H', bpb, 14)[0]
    num_fats = bpb[16]

    fat_sz32 = struct.unpack_from('<I', bpb, 36)[0]
    root_clus = struct.unpack_from('<I', bpb, 44)[0]

    print(f'BPB: bps={bytes_per_sec}, spc={sec_per_clus}, rsvd={rsvd_sec_cnt}, fats={num_fats}')
    print(f'FAT size: {fat_sz32} sectors, root cluster: {root_clus}')

    data_start = rsvd_sec_cnt + (num_fats * fat_sz32)
    data_start_bytes = esp_off + data_start * bytes_per_sec
    print(f'Data area: sector {data_start}, offset {hex(data_start_bytes)}')

    root_off = data_start_bytes + (root_clus - 2) * bytes_per_sec * sec_per_clus
    f.seek(root_off)
    root_data = f.read(bytes_per_sec * sec_per_clus)
    print(f'Root dir at {hex(root_off)}, reading {len(root_data)} bytes')

    for i in range(0, len(root_data), 32):
        entry = root_data[i:i+32]
        if entry[0] == 0:
            break
        if entry[0] == 0xE5:
            continue
        if entry[11] == 0x0F:
            continue
        name = entry[0:8].rstrip(b' ').decode('ascii', errors='replace')
        ext = entry[8:11].rstrip(b' ').decode('ascii', errors='replace')
        full = (name + '.' + ext) if ext else name
        attrs = entry[11]
        clus_hi = struct.unpack_from('<H', entry, 20)[0]
        clus_lo = struct.unpack_from('<H', entry, 26)[0]
        cluster = (clus_hi << 16) | clus_lo
        size = struct.unpack_from('<I', entry, 28)[0]
        is_dir = bool(attrs & 0x10)
        print(f'  Entry: "{full}" cluster={cluster} size={size} dir={is_dir}')
