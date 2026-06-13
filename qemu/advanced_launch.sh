#!/bin/bash
# Script de lanzamiento avanzado para CRONOS W-OS en QEMU

# Configuración avanzada
QEMU="qemu-system-x86_64"
KERNEL_IMAGE="cronos_w-os.img"
OVMF_BIOS="/usr/share/ovmf/OVMF.fd"

# Configuración de hardware avanzado
CPU_TYPE="host"
CPU_FEATURES="+vmx,+aes,+rdrand,+sse3,+sse4.1,+sse4.2,+popcnt,+avx,+avx2,+fma"
CPU_CORES=8
MEMORY_SIZE="32G"

# Configuración de GPU avanzada
GPU_RESOLUTION="2560x1440"
GPU_VRAM="4G"
GPU_TYPE="virtio-gpu-pci"

# Configuración de red avanzada
NETWORK_MAC="52:54:00:12:34:56"
NETWORK_FORWARD="tcp::2222-:22,tcp::8080-:8080"

# Configuración de almacenamiento avanzada
STORAGE_SIZE="1T"
STORAGE_TYPE="nvme"

# Configuración USB avanzada
USB_CONTROLLER="xhci"

# Configuración de debugging avanzado
DEBUG_PORT=1234
SERIAL_OUTPUT="stdio"
LOG_FILE="qemu_advanced.log"

# Función de ayuda avanzada
show_help() {
    echo "🚀 Script de lanzamiento avanzado para CRONOS W-OS"
    echo ""
    echo "Uso: $0 [OPCIONES]"
    echo ""
    echo "Opciones avanzadas:"
    echo "  -h, --help          Muestra esta ayuda"
    echo "  -d, --debug         Activa modo debug completo"
    echo "  -g, --gdb           Activa GDB server"
    echo "  -t, --test          Modo testing (auto-exit)"
    echo "  -m, --memory SIZE   Tamaño de memoria (default: 32G)"
    echo "  -c, --cores NUM     Número de cores (default: 8)"
    echo "  -r, --resolution    Resolución de pantalla (default: 2560x1440)"
    echo "  -b, --bios PATH     Ruta al BIOS UEFI (default: /usr/share/ovmf/OVMF.fd)"
    echo "  --no-gpu            Desactiva GPU virtual"
    echo "  --no-network        Desactiva red virtual"
    echo "  --no-usb            Desactiva USB virtual"
    echo "  --uefi              Activa modo UEFI"
    echo ""
    echo "Ejemplos:"
    echo "  $0                  Lanza CRONOS W-OS en modo avanzado"
    echo "  $0 -d -g            Lanza con debug completo y GDB"
    echo "  $0 --uefi           Lanza en modo UEFI"
    echo "  $0 -m 64G -c 16     Lanza con 64GB RAM y 16 cores"
}

# Parsear argumentos
DEBUG_MODE=false
GDB_MODE=false
TEST_MODE=false
UEFI_MODE=false
NO_GPU=false
NO_NETWORK=false
NO_USB=false

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -d|--debug)
            DEBUG_MODE=true
            shift
            ;;
        -g|--gdb)
            GDB_MODE=true
            shift
            ;;
        -t|--test)
            TEST_MODE=true
            shift
            ;;
        -m|--memory)
            MEMORY_SIZE="$2"
            shift 2
            ;;
        -c|--cores)
            CPU_CORES="$2"
            shift 2
            ;;
        -r|--resolution)
            GPU_RESOLUTION="$2"
            shift 2
            ;;
        -b|--bios)
            OVMF_BIOS="$2"
            shift 2
            ;;
        --uefi)
            UEFI_MODE=true
            shift
            ;;
        --no-gpu)
            NO_GPU=true
            shift
            ;;
        --no-network)
            NO_NETWORK=true
            shift
            ;;
        --no-usb)
            NO_USB=true
            shift
            ;;
        *)
            echo "Opción desconocida: $1"
            show_help
            exit 1
            ;;
    esac
done

# Verificar archivos necesarios
if [ ! -f "$KERNEL_IMAGE" ]; then
    echo "❌ Imagen de kernel no encontrada: $KERNEL_IMAGE"
    echo "💡 Ejecuta 'make image' primero"
    exit 1
fi

if [ "$UEFI_MODE" = true ]; then
    if [ ! -f "$OVMF_BIOS" ]; then
        echo "❌ BIOS UEFI no encontrado: $OVMF_BIOS"
        echo "💡 Instala ovmf: sudo apt install ovmf"
        exit 1
    fi
fi

# Verificar QEMU
if ! command -v $QEMU &> /dev/null; then
    echo "❌ QEMU no encontrado"
    echo "💡 Instala QEMU: sudo apt install qemu-system-x86"
    exit 1
fi

# Construir comando QEMU
QEMU_CMD="$QEMU"

# Configuración básica
if [ "$UEFI_MODE" = true ]; then
    QEMU_CMD="$QEMU_CMD -bios $OVMF_BIOS"
fi

QEMU_CMD="$QEMU_CMD -drive format=raw,file=$KERNEL_IMAGE"
QEMU_CMD="$QEMU_CMD -m $MEMORY_SIZE"
QEMU_CMD="$QEMU_CMD -smp $CPU_CORES"
QEMU_CMD="$QEMU_CMD -cpu $CPU_TYPE,$CPU_FEATURES"

# GPU virtual
if [ "$NO_GPU" = false ]; then
    QEMU_CMD="$QEMU_CMD -device $GPU_TYPE,xres=${GPU_RESOLUTION%,*},yres=${GPU_RESOLUTION#*,},vgmem=$GPU_VRAM"
fi

# Red virtual
if [ "$NO_NETWORK" = false ]; then
    QEMU_CMD="$QEMU_CMD -device virtio-net-pci,netdev=net0,mac=$NETWORK_MAC"
    QEMU_CMD="$QEMU_CMD -netdev user,id=net0,hostfwd=$NETWORK_FORWARD"
fi

# USB virtual
if [ "$NO_USB" = false ]; then
    QEMU_CMD="$QEMU_CMD -device $USB_CONTROLLER,id=usb"
    QEMU_CMD="$QEMU_CMD -device usb-tablet"
    QEMU_CMD="$QEMU_CMD -device usb-kbd"
fi

# Monitor QEMU
QEMU_CMD="$QEMU_CMD -monitor telnet:localhost:$DEBUG_PORT,server,nowait"

# Salida serial
QEMU_CMD="$QEMU_CMD -serial $SERIAL_OUTPUT"

# Logging
QEMU_CMD="$QEMU_CMD -D $LOG_FILE"

# Debug options
if [ "$DEBUG_MODE" = true ]; then
    QEMU_CMD="$QEMU_CMD -d int,cpu_reset,exec,guest_errors,unimp"
fi

if [ "$GDB_MODE" = true ]; then
    QEMU_CMD="$QEMU_CMD -s -S"
fi

# No reboot (para testing)
if [ "$TEST_MODE" = true ]; then
    QEMU_CMD="$QEMU_CMD -no-reboot"
    QEMU_CMD="$QEMU_CMD -device isa-debug-exit,iobase=0xf4,iosize=0x04"
fi

# Mostrar configuración
echo "🚀 Lanzando CRONOS W-OS en QEMU (Modo Avanzado)"
echo "📊 Configuración:"
echo "   💾 Memoria: $MEMORY_SIZE"
echo "   🔥 CPU: $CPU_CORES cores ($CPU_TYPE)"
echo "   🎮 GPU: $GPU_RESOLUTION @ $GPU_VRAM"
echo "   🌐 Red: Activada (MAC: $NETWORK_MAC)"
echo "   🔌 USB: Activado"
if [ "$UEFI_MODE" = true ]; then
    echo "   🖥️ BIOS: $OVMF_BIOS (UEFI)"
fi
echo ""

if [ "$DEBUG_MODE" = true ]; then
    echo "🐛 Modo debug completo activado"
fi

if [ "$GDB_MODE" = true ]; then
    echo "🔍 GDB server activado en localhost:1234"
    echo "💡 Conecta con: gdb -ex 'target remote localhost:1234'"
fi

if [ "$TEST_MODE" = true ]; then
    echo "🧪 Modo test activado (auto-exit)"
fi

echo ""
echo "⚡ Ejecutando:"
echo "$QEMU_CMD"
echo ""

# Ejecutar QEMU
eval $QEMU_CMD

# Limpiar al salir
echo ""
echo "🧹 Limpiando procesos..."
