# PowerShell script to create a bootable GPT disk image using Windows VHD tools
$ErrorActionPreference = "Stop"

$WORK = "F:\cronos_w-os"
$KERNEL = "$WORK\target\x86_64-unknown-none\release\cronos_w_os"
$LIMINE_CFG = "$WORK\limine.cfg"
$LIMINE_DIR = "$WORK\limine-tools\limine-binary"
$LIMINE_EXE = "$LIMINE_DIR\limine-tool-windows-x86\limine.exe"
$BOOTX64 = "$LIMINE_DIR\BOOTX64.EFI"
$OUTPUT = "$WORK\cronos_w-os.img"
$VHD_PATH = "$WORK\cronos_w-os.vhdx"

$img_mb = 64
$img_size = $img_mb * 1024 * 1024

Write-Host "[BUILD] Creating VHD of $img_mb MB..."

# Create VHD
$disk = New-VHD -Path $VHD_PATH -SizeBytes $img_size -Dynamic

# Mount VHD
Mount-VHD -Path $VHD_PATH

# Get disk number
$diskNumber = (Get-DiskImage -ImagePath $VHD_PATH).Number

# Initialize disk as GPT
Initialize-Disk -Number $diskNumber -PartitionStyle GPT

# Create ESP partition (EFI System Partition)
$espPartition = New-Partition -DiskNumber $diskNumber -Size 63MB -GptType '{C12A7328-F81F-11D2-BA4B-00A0C93EC93B}' -AssignDriveLetter

# Format as FAT32
Format-Volume -DriveLetter $espPartition.DriveLetter -FileSystem FAT32 -NewFileSystemLabel "CRONOS ESP" -Confirm:$false

# Create directory structure
$drive = "${espPartition.DriveLetter}:\"
New-Item -ItemType Directory -Force -Path "$drive\EFI"
New-Item -ItemType Directory -Force -Path "$drive\EFI\BOOT"

# Copy files
Write-Host "[BUILD] Copying files..."
Copy-Item -Path $BOOTX64 -Destination "$drive\EFI\BOOT\BOOTX64.EFI"
Copy-Item -Path $KERNEL -Destination "$drive\cronos_w_os"
Copy-Item -Path $LIMINE_CFG -Destination "$drive\limine.cfg"

# Dismount VHD
Dismount-VHD -Path $VHD_PATH

# Convert VHD to raw image
Write-Host "[BUILD] Converting VHD to raw image..."
$bytes = [System.IO.File]::ReadAllBytes($VHD_PATH)
[System.IO.File]::WriteAllBytes($OUTPUT, $bytes)

# Cleanup
Remove-Item -Path $VHD_PATH -Force

Write-Host "[BUILD] Done! Image: $OUTPUT"

# Install Limine bootloader
Write-Host "[BUILD] Installing Limine bootloader..."
$result = & $LIMINE_EXE "bios-install" $OUTPUT 2>&1
if ($LASTEXITCODE -ne 0) {
    Write-Host "[WARN] limine install failed: $result"
} else {
    Write-Host "  limine install OK"
}
