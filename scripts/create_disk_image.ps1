# Script PowerShell para crear imagen de disco bootable de CRONOS W-OS con Limine
# FASE 15: Bootloader and Boot Process

# Configuration
$KernelPath = "target\x86_64-unknown-none\release\cronos_w_os"
$LimineCfg = "limine.cfg"
$OutputImage = "cronos_w-os.img"
$LimineDir = "limine-tools\limine-binary"
$LimineExe = "$LimineDir\limine-tool-windows-x86\limine.exe"

# Image size (100 MB)
$ImageSize = 100 * 1024 * 1024

Write-Host "Creating bootable disk image with Limine..." -ForegroundColor Green

# Check if kernel exists
if (-not (Test-Path $KernelPath)) {
    Write-Host "Error: Kernel not found at $KernelPath" -ForegroundColor Red
    Write-Host "Run 'cargo build --release --target x86_64-unknown-none' first" -ForegroundColor Yellow
    exit 1
}

# Check if limine.cfg exists
if (-not (Test-Path $LimineCfg)) {
    Write-Host "Error: limine.cfg not found" -ForegroundColor Red
    exit 1
}

# Create temporary directory for partition
$TempDir = "temp_disk"
if (Test-Path $TempDir) {
    Remove-Item -Recurse -Force $TempDir
}
New-Item -ItemType Directory -Path $TempDir -Force | Out-Null

# Create EFI directory
$EfiDir = "$TempDir\EFI\BOOT"
New-Item -ItemType Directory -Path $EfiDir -Force | Out-Null

# Copy kernel to partition
$KernelDest = "$TempDir\cronos_w_os"
Copy-Item $KernelPath $KernelDest

# Copy limine.cfg
Copy-Item $LimineCfg "$TempDir\limine.cfg"

# Copy Limine files for BIOS
$LimineBios = "$LimineDir\limine-bios.sys"
if (Test-Path $LimineBios) {
    Copy-Item $LimineBios "$TempDir\limine-bios.sys"
}

# Copy Limine files for UEFI
$LimineUefi = "$LimineDir\BOOTX64.EFI"
if (Test-Path $LimineUefi) {
    Copy-Item $LimineUefi "$EfiDir\BOOTX64.EFI"
}

# Create disk image
Write-Host "Creating disk image of $($ImageSize / 1MB) MB..." -ForegroundColor Cyan
$ImageData = [byte[]]::new($ImageSize)
[System.IO.File]::WriteAllBytes($OutputImage, $ImageData)

# Use limine.exe to install bootloader if available
if (Test-Path $LimineExe) {
    Write-Host "Installing Limine bootloader..." -ForegroundColor Cyan
    & $LimineExe bios-install $OutputImage
} else {
    Write-Host "Limine tool not found, creating basic image..." -ForegroundColor Yellow
}

# Clean temporary directory
Remove-Item -Recurse -Force $TempDir

Write-Host "Image created: $OutputImage" -ForegroundColor Green
Write-Host ""
Write-Host "To boot in QEMU:" -ForegroundColor Cyan
Write-Host "  qemu-system-x86_64 -drive format=raw,file=$OutputImage -m 2G -serial stdio"
Write-Host ""
Write-Host "To boot on real hardware:" -ForegroundColor Cyan
Write-Host "  Use Rufus or similar to flash the image to a USB"
