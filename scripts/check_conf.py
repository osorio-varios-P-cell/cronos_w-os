import struct, sys, io
sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding='utf-8', errors='replace')

with open('F:\\cronos_w-os-github\\cronos_w-os.img', 'rb') as f:
    bps = 512
    reserved_sectors = 9
    num_fats = 2
    sectors_per_fat = 8035
    fat_start = 2048 + reserved_sectors
    data_start = fat_start + num_fats * sectors_per_fat

    f.seek(data_start * bps)  # root dir cluster 2
    root_data = f.read(bps)
    
    print('=== Root directory entries ===')
    for off in range(0, len(root_data), 32):
        entry = root_data[off:off+32]
        if entry[0] == 0:
            print(f'{off:4d}: END')
            break
        if entry[0] == 0xE5:
            print(f'{off:4d}: DELETED')
            continue
        
        attr = entry[11]
        if attr & 0x0F == 0x0F:
            seq = entry[0] & 0x3F
            last = (entry[0] & 0x40) != 0
            # Read the 13 UTF-16LE characters
            chars = []
            for i in [1,3,5,7,9,14,16,18,20,22,24,28,30]:
                val = struct.unpack('<H', entry[i:i+2])[0]
                chars.append(chr(val) if 0x20 <= val < 0xFFFF else '.')
            name = ''.join(chars).rstrip('.')
            print(f'{off:4d}: LFN[seq={seq:2d},last={last:1d}] "{name}"')
        else:
            name = entry[0:8].decode('ascii', errors='replace').strip()
            ext = entry[8:11].decode('ascii', errors='replace').strip()
            if ext:
                name += '.' + ext
            ch = struct.unpack('<H', entry[20:22])[0] << 16 | struct.unpack('<H', entry[26:28])[0]
            sz = struct.unpack('<I', entry[28:32])[0]
            print(f'{off:4d}: SFN "{name:20s}" Cluster={ch:5d} Size={sz}')
