import struct

with open('target/x86_64-unknown-none/release/cronos_w_os', 'rb') as f:
    data = f.read()

ehdr = data[0:64]
e_shoff = struct.unpack('<Q', ehdr[40:48])[0]
e_shentsize = struct.unpack('<H', ehdr[58:60])[0]
e_shnum = struct.unpack('<H', ehdr[60:62])[0]
e_shstrndx = struct.unpack('<H', ehdr[62:64])[0]

print(f'Entry point: 0x{struct.unpack("<Q", ehdr[24:32])[0]:x}')
print(f'Sections: {e_shnum}')
print(f'Shstrndx: {e_shstrndx}')

# Read section string table header
shstr_off = e_shoff + e_shstrndx * e_shentsize
shstr_hdr = data[shstr_off:shstr_off+e_shentsize]
strtab_off = struct.unpack('<Q', shstr_hdr[24:32])[0]
strtab_sz = struct.unpack('<Q', shstr_hdr[32:40])[0]
strtab = data[strtab_off:strtab_off+strtab_sz]

# Print section headers
for i in range(e_shnum):
    off = e_shoff + i * e_shentsize
    shdr = data[off:off+e_shentsize]
    sh_name = struct.unpack('<I', shdr[0:4])[0]
    sh_type = struct.unpack('<I', shdr[4:8])[0]
    sh_flags = struct.unpack('<Q', shdr[8:16])[0]
    sh_addr = struct.unpack('<Q', shdr[16:24])[0]
    sh_offset = struct.unpack('<Q', shdr[24:32])[0]
    sh_size = struct.unpack('<Q', shdr[32:40])[0]
    sh_link = struct.unpack('<I', shdr[40:44])[0]

    name = ''
    j = sh_name
    while j < len(strtab) and strtab[j] != 0:
        name += chr(strtab[j])
        j += 1

    if name:
        print(f'  [{i:2d}] {name:25s} type={sh_type:3d} flags=0x{sh_flags:08x} addr=0x{sh_addr:016x} offset=0x{sh_offset:x} size={sh_size}')
