import struct

with open('cronos_w-os.img', 'rb') as f:
    esp_off = 162 * 512
    f.seek(esp_off)
    bpb = f.read(512)

    fat_sz = struct.unpack_from('<H', bpb, 22)[0]
    rsvd = struct.unpack_from('<H', bpb, 14)[0]
    fats = bpb[16]
    root_entries = struct.unpack_from('<H', bpb, 17)[0]
    bps = struct.unpack_from('<H', bpb, 11)[0]
    spc = bpb[13]

    root_dir_secs = (root_entries * 32 + bps - 1) // bps
    data_start = rsvd + (fats * fat_sz) + root_dir_secs
    root_dir_off = esp_off + (rsvd + fats * fat_sz) * bps
    data_off = esp_off + data_start * bps

    print(f'FAT16: rsvd={rsvd}, fats={fats}, fat_sz={fat_sz}, root_entries={root_entries}')
    print(f'Root dir offset: {hex(root_dir_off)}')
    print(f'Data start sector: {data_start}, offset: {hex(data_off)}')

    f.seek(root_dir_off)
    root_data = f.read(root_dir_secs * bps)

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
        cluster = struct.unpack_from('<H', entry, 26)[0]
        size = struct.unpack_from('<I', entry, 28)[0]
        is_dir = bool(entry[11] & 0x10)
        is_vol = bool(entry[11] & 0x08)
        print(f'  {i//32:3d}: "{full}" cluster={cluster} size={size} dir={is_dir} vol={is_vol}')
