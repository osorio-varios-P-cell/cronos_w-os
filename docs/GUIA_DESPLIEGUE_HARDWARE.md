# Guía de Despliegue en Hardware Real: Protocolo GENESIS v3.3

Esta guía detalla los pasos para transformar el código fuente de Cronos OS en un sistema operativo funcional en tu computadora física.

## 1. Requisitos de Hardware
- **Procesador:** x86_64 con soporte para UEFI.
- **Memoria RAM:** Mínimo 4 GB (16 GB recomendado para Modo Fluido y VMs).
- **Almacenamiento:** USB de al menos 1 GB.
- **Firmware:** Secure Boot debe estar **desactivado**.

## 2. Preparación de la Imagen (`cronos_w-os.img`)

Si estás en un entorno de desarrollo con las herramientas de compilación instaladas:

```bash
# 1. Compilar el kernel en modo release para máxima estabilidad
cargo build --release --target x86_64-unknown-none

# 2. Generar la imagen de disco con el bootloader Limine
# (Este paso requiere el script de empaquetado del repositorio)
python3 scripts/create_boot_image.py
```

## 3. Grabación en USB (Protocolo de Integridad)

**¡ADVERTENCIA!** El comando `dd` borrará todos los datos del disco de destino.

### En Linux/macOS:
```bash
# Identifica tu USB (ej. /dev/sdX o /dev/diskX)
lsblk  # En Linux
diskutil list # En macOS

# Grabar la imagen
sudo dd if=cronos_w-os.img of=/dev/sdX bs=4M status=progress conv=fsync
```

### En Windows:
Usa herramientas como **Rufus** o **BalenaEtcher** seleccionando el archivo `cronos_w-os.img` y tu unidad USB.

## 4. El Primer Arranque (Secuencia de Vida)

1. Inserta el USB y enciende la computadora.
2. Pulsa la tecla de selección de arranque (F12, F11, F10 o Esc según tu placa base).
3. Selecciona la opción de **UEFI: [Nombre de tu USB]**.
4. Verás el menú de Limine. Selecciona **CRONOS W-OS**.

## 5. Estabilidad y Recuperación (Modo Murphy)

Cronos OS incluye un **Installer Ledger** para diagnosticar fallos:

- **Si el arranque se detiene:** Observa los mensajes seriales (si tienes un cable serial conectado) o la pantalla de debug de Limine.
- **Safe Mode:** Si un hardware causa pánico, Cronos lo registrará en el grafo. En el segundo arranque, Hive AI intentará omitir el driver problemático.
- **Shell Soberana:** Una vez en el escritorio, usa el comando `status` para verificar que el Grafo de Conocimiento haya mapeado correctamente tu RAM y CPU.

## 6. Comandos Iniciales Recomendados
Al ver el prompt `@sovereign:#`, ejecuta:
1. `sysinfo`: Para confirmar la detección de núcleos y memoria.
2. `brain-init`: Para vincular tus archivos Markdown de Obsidian.
3. `fable`: Para que la IA analice y optimice el consumo de energía de tu hardware real.
