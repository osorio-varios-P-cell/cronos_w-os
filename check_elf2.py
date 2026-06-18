import struct

with open('target/x86_64-unknown-none/release/cronos_w_os', 'rb') as f:
    data = f.read()

ehdr = data[0:64]
e_shoff = struct.unpack('<Q', ehdr[40:48])[0]
e_shentsize = struct.unpack('<H', ehdr[58:60])[0]
e_shnum = struct.unpack('<H', ehdr[60:62])[0]
e_shstrndx = struct.unpack('<H', ehdr[62:64])[0]

# Read section headers
sections = []
for i in range(e_shnum):
    off = e_shoff + i * e_shentsize
    shdr = data[off:off+e_shentsize]
    sections.append({
        'name_idx': struct.unpack('<I', shdr[0:4])[0],
        'type': struct.unpack('<I', shdr[4:8])[0],
        'flags': struct.unpack('<Q', shdr[8:16])[0],
        'addr': struct.unpack('<Q', shdr[16:24])[0],
        'offset': struct.unpack('<Q', shdr[24:32])[0],
        'size': struct.unpack('<Q', shdr[32:40])[0],
        'link': struct.unpack('<I', shdr[40:44])[0],
        'info': struct.unpack('<I', shdr[44:48])[0],
        'addralign': struct.unpack('<Q', shdr[48:56])[0],
        'entsize': struct.unpack('<Q', shdr[56:64])[0],
    })

# Read string table for section names
strtab_hdr = sections[e_shstrndx]
strtab = data[strtab_hdr['offset']:strtab_hdr['offset']+strtab_hdr['size']]

def get_name(name_idx):
    s = ''
    j = name_idx
    while j < len(strtab) and strtab[j] != 0:
        s += chr(strtab[j])
        j += 1
    return s

# Map section index to name
section_names = {i: get_name(s['name_idx']) for i, s in enumerate(sections)}

# Check .limine_requests content
for i, s in enumerate(sections):
    name = section_names[i]
    if name == '.limine_requests':
        print(f'=== .limine_requests section ({s["size"]} bytes) ===')
        req_data = data[s['offset']:s['offset']+s['size']]
        print(f'Raw hex:')
        for k in range(0, len(req_data), 16):
            hex_str = ' '.join(f'{b:02x}' for b in req_data[k:k+16])
            print(f'  {k:04x}: {hex_str}')
    if name == '.symtab':
        symtab = data[s['offset']:s['offset']+s['size']]
        entsize = s['entsize'] if s['entsize'] else 24
        strtab_idx = s['link']
        strtab_sec = sections[strtab_idx]
        sym_strtab = data[strtab_sec['offset']:strtab_sec['offset']+strtab_sec['size']]
        print(f'\n=== Symbol Table ({s["size"]} bytes, {entsize} per entry) ===')
        numsyms = s['size'] // entsize
        for k in range(numsyms):
            sym = symtab[k*entsize:(k+1)*entsize]
            st_name = struct.unpack('<I', sym[0:4])[0]
            st_value = struct.unpack('<Q', sym[8:16])[0]
            st_size = struct.unpack('<Q', sym[16:24])[0]
            st_info = sym[4]
            st_bind = st_info >> 4
            st_type = st_info & 0xf
            # Get name
            sname = ''
            j = st_name
            while j < len(sym_strtab) and sym_strtab[j] != 0:
                sname += chr(sym_strtab[j])
                j += 1
            if sname in ('_start', 'kmain', 'kernel_main'):
                print(f'  {k:3d}: name="{sname}" bind={st_bind} type={st_type} value=0x{st_value:016x} size={st_size}')
