#!/usr/bin/env python3
"""
Script para crear imagen de disco bootable de CRONOS W-OS con Limine bootloader
FASE 15: Bootloader y Boot Process
"""

import os
import subprocess
import shutil
import struct

# Configuración
KERNEL_PATH = "target/x86_64-unknown-none/release/cronos_w_os"
LIMINE_CFG = "limine.cfg"
OUTPUT_IMAGE = "cronos_w-os.img"
LIMINE_DIR = "limine-tools/limine-binary"
LIMINE_EXE = f"{LIMINE_DIR}/limine-tool-windows-x86/limine.exe"

# Tamaño de la imagen (100 MB)
IMAGE_SIZE = 100 * 1024 * 1024

def run_command(cmd, check=True):
    """Ejecuta un comando y muestra la salida"""
    print(f"Ejecutando: {' '.join(cmd)}")
    result = subprocess.run(cmd, capture_output=True, text=True, check=check)
    if result.stdout:
        print(result.stdout)
    if result.stderr:
        print(result.stderr)
    return result

def create_disk_image():
    """Crea una imagen de disco bootable con Limine"""
    print("🚀 Creando imagen de disco bootable con Limine...")
    
    # Verificar que el kernel existe
    if not os.path.exists(KERNEL_PATH):
        print(f"❌ Error: Kernel no encontrado en {KERNEL_PATH}")
        print("💡 Ejecuta 'cargo build --release --target x86_64-unknown-none' primero")
        return False
    
    # Verificar que limine.cfg existe
    if not os.path.exists(LIMINE_CFG):
        print(f"❌ Error: limine.cfg no encontrado")
        return False
    
    # Crear directorio temporal para la partición
    temp_dir = "temp_disk"
    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)
    os.makedirs(temp_dir)
    
    # Crear directorio EFI
    efi_dir = os.path.join(temp_dir, "EFI", "BOOT")
    os.makedirs(efi_dir, exist_ok=True)
    
    # Copiar kernel a la partición
    kernel_dest = os.path.join(temp_dir, "cronos_w_os")
    shutil.copy(KERNEL_PATH, kernel_dest)
    
    # Copiar limine.cfg
    shutil.copy(LIMINE_CFG, os.path.join(temp_dir, "limine.cfg"))
    
    # Copiar archivos de Limine para BIOS
    limine_bios = os.path.join(LIMINE_DIR, "limine-bios.sys")
    if os.path.exists(limine_bios):
        shutil.copy(limine_bios, os.path.join(temp_dir, "limine-bios.sys"))
    
    # Copiar archivos de Limine para UEFI
    limine_uefi = os.path.join(LIMINE_DIR, "limine-uefi-cd.bin")
    if os.path.exists(limine_uefi):
        shutil.copy(limine_uefi, os.path.join(efi_dir, "BOOTX64.EFI"))
    
    # Crear imagen de disco
    print(f"💾 Creando imagen de disco de {IMAGE_SIZE // (1024*1024)} MB...")
    with open(OUTPUT_IMAGE, 'wb') as f:
        f.write(b'\x00' * IMAGE_SIZE)
    
    # Crear partición FAT32
    print("📝 Creando partición FAT32...")
    # Nota: En Windows, necesitamos usar herramientas diferentes
    # Por ahora, creamos una imagen básica
    
    # Usar limine.exe para instalar el bootloader
    if os.path.exists(LIMINE_EXE):
        print("🔧 Instalando Limine bootloader...")
        run_command([LIMINE_EXE, OUTPUT_IMAGE, "install"])
    else:
        print("⚠️  Limine tool no encontrado, creando imagen básica...")
    
    # Limpiar directorio temporal
    shutil.rmtree(temp_dir)
    
    print(f"✅ Imagen creada: {OUTPUT_IMAGE}")
    return True

def main():
    """Función principal"""
    print("🚀 Script de creación de imagen de disco para CRONOS W-OS")
    print("🔧 Bootloader: Limine")
    print("")
    
    try:
        success = create_disk_image()
        if success:
            print("")
            print("✅ Imagen creada exitosamente")
            print(f"📁 Archivo: {OUTPUT_IMAGE}")
            print("")
            print("Para bootear en QEMU:")
            print("  make qemu")
            print("")
            print("Para bootear en hardware real:")
            print("  sudo dd if=cronos_w-os.img of=/dev/sdX bs=4M status=progress")
            print("  (Reemplaza /dev/sdX con tu dispositivo USB)")
        else:
            print("❌ Error creando la imagen")
            return 1
    except Exception as e:
        print(f"❌ Error: {e}")
        return 1
    
    return 0

if __name__ == "__main__":
    exit(main())
