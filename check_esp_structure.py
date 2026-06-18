from FATtools import fat
import io

img = open('F:\\cronos_w-os-github\\cronos_w-os.img', 'rb')
img.seek(98*512)  # ESP partition starts at LBA 98
esp_data = img.read(131072*512)  # 64 MB ESP partition
fs = fat.FATFileSystem(esp_data)

print('Root directory:')
for item in fs.listdir('/'):
    print(f'  {item}')

print('\nEFI directory:')
try:
    for item in fs.listdir('/EFI'):
        print(f'  {item}')
except:
    print('  /EFI not found')

print('\nEFI/BOOT directory:')
try:
    for item in fs.listdir('/EFI/BOOT'):
        print(f'  {item}')
except:
    print('  /EFI/BOOT not found')
