import struct

with open('cronos_w-os.img', 'rb') as f:
    mbr = f.read(512)
    print(f'MBR signature: {mbr[510]:02x} {mbr[511]:02x}')
    print(f'Partition type at 0x1C2: {mbr[0x1C2]:02x}')
    
    gpt = f.read(512)
    print(f'GPT signature: {gpt[0:8]}')
    if gpt[0:8] == b'EFI PART':
        usable_start = struct.unpack('<Q', gpt[40:48])[0]
        usable_end = struct.unpack('<Q', gpt[48:56])[0]
        disk_guid = gpt[56:72]
        pe_start = struct.unpack('<Q', gpt[72:80])[0]
        pe_count = struct.unpack('<I', gpt[80:84])[0]
        pe_size = struct.unpack('<I', gpt[84:88])[0]
        print(f'First usable LBA: {usable_start}')
        print(f'Last usable LBA: {usable_end}')
        print(f'Partition entries: start={pe_start}, count={pe_count}, size={pe_size}')
        
        f.seek(pe_start * 512)
        for i in range(min(pe_count, 8)):
            entry = f.read(pe_size)
            type_guid = entry[:16]
            start_lba = struct.unpack('<Q', entry[32:40])[0]
            end_lba = struct.unpack('<Q', entry[40:48])[0]
            name = entry[56:128].rstrip(b'\x00').decode('utf-16-le', errors='ignore')
            if start_lba != 0:
                print(f'Partition {i}: type={type_guid.hex()}, start={start_lba}, count={end_lba-start_lba+1}, name="{name}"')
