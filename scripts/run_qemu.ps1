$ErrorActionPreference = "Stop"
$qemu = "C:\Program Files\qemu\qemu-system-x86_64w.exe"
$bios = "C:\Program Files\qemu\share\edk2-x86_64-code.fd"
$image = "cronos_w-os.img"
$serial_file = Join-Path (Get-Location) "qemu_serial.txt"
$debug_file = Join-Path (Get-Location) "qemu_debug.txt"

# Remove old files
if (Test-Path $serial_file) { Remove-Item -Force $serial_file }

# Build argument list (array to handle spaces)
$args = @(
    "-drive", "if=pflash,format=raw,readonly=on,file=$bios",
    "-drive", "format=raw,file=$image",
    "-m", "2G",
    "-smp", "2",
    "-cpu", "qemu64",
    "-net", "none",
    "-display", "none",
    "-no-reboot",
    "-D", $debug_file,
    "-d", "int,cpu_reset"
)

# Use a named pipe for serial output
$pipeName = "cronos_serial"
$pipePath = "\\.\pipe\$pipeName"

# Create named pipe server in a background job
$job = Start-Job -ScriptBlock {
    param($pipeName, $outputFile)
    $pipe = New-Object System.IO.Pipes.NamedPipeServerStream($pipeName, [System.IO.Pipes.PipeDirection]::In, 1, [System.IO.Pipes.PipeTransmissionMode]::Byte)
    $pipe.WaitForConnection()
    $reader = New-Object System.IO.StreamReader($pipe)
    $writer = New-Object System.IO.StreamWriter($outputFile)
    while (($line = $reader.ReadLine()) -ne $null) {
        $writer.WriteLine($line)
    }
    $reader.Close()
    $pipe.Close()
    $writer.Close()
} -ArgumentList $pipeName, $serial_file

Start-Sleep -Seconds 1

# Start QEMU
$qemuArgs = $args + @("-serial", "pipe:$pipeName")
Write-Output "Starting QEMU..."
$proc = Start-Process -FilePath $qemu -ArgumentList $qemuArgs -NoNewWindow -PassThru

Start-Sleep -Seconds 20

if (!$proc.HasExited) {
    $proc.Kill()
    Write-Output "TIMEOUT after 20 seconds"
} else {
    Write-Output "QEMU exited with code: $($proc.ExitCode)"
}

Stop-Job $job -ErrorAction SilentlyContinue
Remove-Job $job -ErrorAction SilentlyContinue
