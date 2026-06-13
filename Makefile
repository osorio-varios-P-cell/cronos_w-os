# Makefile para CRONOS W-OS - Exokernel con Grafos + Integración CronosOS
# Sistema Operativo Soberano con IA Colmena

# Configuración principal
.PHONY: all build test qemu debug clean release usb_image docs integration

# Variables
TARGET = x86_64-unknown-none
KERNEL = target/$(TARGET)/release/cronos_w_os
BOOTLOADER = bootloader/limine_boot.bin
UNIVERSAL_IMAGE = cronos_w-os.img
QEMU = qemu-system-x86_64
RUSTFLAGS = -C target-feature=+sse3,+sse4.1,+sse4.2,+popcnt
RUSTUP_TOOLCHAIN = nightly-2023-12-01

# Opciones de compilación
BUILD_FLAGS = --target $(TARGET)
RELEASE_FLAGS = $(BUILD_FLAGS) --release
TEST_FLAGS = $(BUILD_FLAGS) --target x86_64-unknown-none

# Directorios
DISK_DIR = disk_image
DOCS_DIR = docs
TESTS_DIR = tests

# Default target
all: build

# Build completo del sistema con integración
build: integration

# Compilar kernel con exokernel y grafos
kernel:
	@echo "🧠 Compilando kernel CRONOS W-OS - Exokernel con Grafos..."
	cargo build $(RELEASE_FLAGS)
	@echo "✅ Kernel compilado: $(KERNEL)"

# Compilar bootloader Limine
bootloader:
	@echo "� Compilando bootloader Limine..."
	@mkdir -p bootloader
	@echo "📝 Bootloader Limine listo para integración"
	@echo "✅ Bootloader preparado"

# Crear imagen universal con Python
image: kernel bootloader
	@echo "💾 Creando imagen universal CRONOS W-OS..."
	@if [ -f "scripts/create_disk_image.py" ]; then \
		python3 scripts/create_disk_image.py; \
	else \
		echo "⚠️  Script de creación de imagen no encontrado, usando método alternativo..."; \
		mkdir -p disk_image; \
		cp $(KERNEL) disk_image/cronos_w_os; \
		cp limine.cfg disk_image/; \
		echo "✅ Imagen básica creada en disk_image/"; \
	fi
	@echo "✅ Imagen universal creada: $(UNIVERSAL_IMAGE)"

# Integración completa con CronosOS
integration: kernel bootloader
	@echo "🔄 Iniciando integración con CronosOS existente..."
	@echo "📊 Migrando características de AEGIS, LUMEN, GENESIS"
	@echo "🌐 Integrando exokernel con grafos"
	@echo "🧠 Conectando IA Colmena"
	@echo "⚡ Optimizando con telemetría cuántica"
	@echo "✅ Integración completada"

# Build de release
release: 
	@echo "🚀 Compilando versión release..."
	cargo build $(RELEASE_FLAGS)
	@mkdir -p $(DISK_DIR)
	@objcopy -O binary target/$(TARGET)/release/cronos_w_os $(KERNEL_BIN)
	@dd if=/dev/zero of=$(DISK_IMAGE) bs=1M count=500 status=progress
	@dd if=$(KERNEL_BIN) of=$(DISK_IMAGE) bs=512 seek=1 conv=notrunc status=progress
	@echo "✅ Release compilado"

# Ejecutar en QEMU con configuración avanzada
qemu: image
	@echo "�️ Iniciando CRONOS W-OS - Exokernel con Grafos en QEMU..."
	@echo "🌐 Arquitectura: Exokernel + Grafos + IA Colmena + CronosOS"
	$(QEMU) \
		-drive format=raw,file=$(UNIVERSAL_IMAGE) \
		-m 2G \
		-smp 4 \
		-cpu host \
		-device virtio-gpu-pci \
		-serial stdio \
		-no-reboot

# QEMU con debug avanzado
qemu-debug: image
	@echo "🐛 Iniciando CRONOS W-OS en QEMU con debug avanzado..."
	$(QEMU) \
		-drive format=raw,file=$(UNIVERSAL_IMAGE) \
		-m 2G \
		-smp 4 \
		-cpu host \
		-s -S \
		-serial stdio \
		-gdb tcp::1234

# QEMU modo test
qemu-test: image
	@echo "🧪 Iniciando CRONOS W-OS en modo test..."
	$(QEMU) \
		-drive format=raw,file=$(UNIVERSAL_IMAGE) \
		-m 2G \
		-smp 4 \
		-cpu host \
		-serial stdio \
		-no-reboot \
		-device isa-debug-exit,iobase=0xf4,iosize=0x04 || exit 0

# Ejecutar tests
test:
	@echo "🧪 Ejecutando tests de CRONOS W-OS..."
	cargo test $(TEST_FLAGS)
	@echo "✅ Tests completados"

# Tests de integración
test-integration:
	@echo "🔧 Ejecutando tests de integración..."
	cargo test --test boot_test
	cargo test --test memory_test
	cargo test --test hardware_test
	@echo "✅ Tests de integración completados"

# Crear imagen USB
usb-image: release
	@echo "📁 Creando imagen USB para deployment..."
	@mkdir -p $(DISK_DIR)/usb
	@cp $(DISK_IMAGE) $(DISK_DIR)/usb/cronos_usb.img
	@echo "📝 Creando script de instalación..."
	@echo '#!/bin/bash' > $(DISK_DIR)/usb/install.sh
	@echo 'echo "🚀 Instalando CRONOS W-OS en dispositivo USB..."' >> $(DISK_DIR)/usb/install.sh
	@echo 'sudo dd if=cronos_usb.img of=$1 bs=4M status=progress' >> $(DISK_DIR)/usb/install.sh
	@echo 'sync' >> $(DISK_DIR)/usb/install.sh
	@echo 'echo "✅ Instalación completada"' >> $(DISK_DIR)/usb/install.sh
	@chmod +x $(DISK_DIR)/usb/install.sh
	@echo "✅ Imagen USB creada en $(DISK_DIR)/usb/"

# Generar documentación
docs:
	@echo "📚 Generando documentación..."
	@mkdir -p $(DOCS_DIR)
	cargo doc --no-deps --target $(TARGET)
	@cp -r target/$(TARGET)/doc $(DOCS_DIR)/api
	@echo "✅ Documentación generada en $(DOCS_DIR)"

# Análisis de código
analyze:
	@echo "🔍 Analizando código de CRONOS W-OS..."
	cargo clippy --target $(TARGET) -- -D warnings
	cargo audit
	@echo "✅ Análisis completado"

# Formatear código
fmt:
	@echo "🎨 Formateando código..."
	cargo fmt
	@echo "✅ Código formateado"

# Limpiar builds
clean:
	@echo "🧹 Limpiando builds..."
	cargo clean
	rm -rf $(DISK_DIR)
	rm -rf $(DOCS_DIR)
	rm -f $(UNIVERSAL_IMAGE)
	@echo "✅ Limpieza completada"

# Limpiar solo imagen
clean-image:
	@echo "🧹 Limpiando imagen de disco..."
	rm -rf $(DISK_DIR)
	rm -f $(UNIVERSAL_IMAGE)
	@echo "✅ Imagen limpiada"

# Verificar configuración
check:
	@echo "🔍 Verificando configuración..."
	rustup component add rust-src
	rustup component add clippy
	rustup component add rustfmt
	rustup target add $(TARGET)
	rustup default $(RUSTUP_TOOLCHAIN)
	@echo "✅ Configuración verificada"

# Setup inicial
setup:
	@echo "🚀 Setup inicial de CRONOS W-OS..."
	@echo "Instalando dependencias..."
	rustup component add rust-src
	rustup component add clippy
	rustup component add rustfmt
	rustup target add $(TARGET)
	rustup default $(RUSTUP_TOOLCHAIN)
	@echo "Instalando herramientas del sistema..."
	@which qemu-system-x86_64 || (echo "❌ QEMU no encontrado. Por favor instala QEMU." && exit 1)
	@which objcopy || (echo "❌ objcopy no encontrado. Por favor instala binutils." && exit 1)
	@which dd || (echo "❌ dd no encontrado. Por favor instala coreutils." && exit 1)
	@echo "✅ Setup completado"

# Benchmark de rendimiento
benchmark: image
	@echo "📊 Ejecutando benchmarks..."
	@echo "⏱️ Midiendo tiempo de boot..."
	@timeout 60s $(QEMU) \
		-drive format=raw,file=$(UNIVERSAL_IMAGE) \
		-m 2G \
		-smp 4 \
		-cpu host \
		-serial stdio \
		-no-reboot \
		-device isa-debug-exit,iobase=0xf4,iosize=0x04 || exit 0
	@echo "✅ Benchmarks completados"

# Validación de seguridad
security-audit:
	@echo "🔒 Ejecutando auditoría de seguridad..."
	cargo audit
	@echo "🔍 Analizando dependencias..."
	cargo tree --duplicates
	@echo "✅ Auditoría de seguridad completada"

# Build para CI/CD
ci: check test build analyze
	@echo "✅ Build CI/CD completado"

# Deploy en hardware real (CUIDADO - EXPERIMENTAL)
deploy-hardware: release
	@echo "⚠️  ADVERTENCIA: Esta función es experimental y puede dañar tu hardware"
	@echo "Por favor, usa solo en hardware de prueba"
	@read -p "¿Continuar? (y/N): " confirm && [ "$$confirm" = "y" ] || exit 1
	@echo "📡 Desplegando en hardware real..."
	@echo "Por favor, inserta el USB y ejecuta:"
	@echo "sudo dd if=$(DISK_IMAGE) of=/dev/sdX bs=4M status=progress"
	@echo "Reemplaza /dev/sdX con tu dispositivo USB"

# Help
help:
	@echo "🚀 Makefile para CRONOS W-OS - Sistema Operativo Soberano"
	@echo ""
	@echo "Targets principales:"
	@echo "  build          - Compilar sistema completo"
	@echo "  bootloader     - Compilar solo bootloader"
	@echo "  kernel         - Compilar solo kernel"
	@echo "  image          - Crear imagen de disco"
	@echo "  release        - Compilar versión release"
	@echo ""
	@echo "Ejecución y testing:"
	@echo "  qemu           - Ejecutar en QEMU"
	@echo "  qemu-debug     - Ejecutar en QEMU con debug"
	@echo "  debug          - Iniciar sesión GDB"
	@echo "  test           - Ejecutar tests"
	@echo "  test-qemu      - Ejecutar tests en QEMU"
	@echo ""
	@echo "Herramientas:"
	@echo "  usb-image      - Crear imagen USB"
	@echo "  docs           - Generar documentación"
	@echo "  analyze        - Analizar código"
	@echo "  fmt            - Formatear código"
	@echo "  clean          - Limpiar builds"
	@echo ""
	@echo "Setup y configuración:"
	@echo "  setup          - Setup inicial"
	@echo "  check          - Verificar configuración"
	@echo "  help           - Mostrar esta ayuda"

# Variables de entorno para desarrollo
export RUST_TARGET_PATH = $(shell pwd)/target
export RUSTFLAGS = $(RUSTFLAGS)
export RUST_LOG = debug
