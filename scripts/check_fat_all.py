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
    root_dir_secs = (root_entries * 32 + bps - 1) // bps
    root_dir_off = esp_off + (rsvd + fats * fat_sz) * bps

    f.seek(root_dir_off)
    root_data = f.read(root_dir_secs * bps)

    print("ALL root directory entries (including VFAT LFN):")
    for i in range(0, len(root_data), 32):
        entry = root_data[i:i+32]
        if entry[0] == 0:
            print(f"  {i//32:3d}: [END]")
            break
        if entry[0] == 0xE5:
            print(f"  {i//32:3d}: [DELETED]")
            continue
        attr = entry[11]
        if attr == 0x0F:
            # VFAT LFN entry
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
            cluster = struct.unpack_from('<H', entry, 26)[0]
            size = struct.unpack_from('<I', entry, 28)[0]
            is_dir = bool(attr & 0x10)
            is_vol = bool(attr & 0x08)
            print(f"  {i//32:3d}: SFN=\"{full}\" cluster={cluster} size={size} dir={is_dir} vol={is_vol}")
