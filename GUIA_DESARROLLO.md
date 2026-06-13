# Guía de Desarrollo de CRONOS W-OS

## 📋 Tabla de Contenidos

1. [Configuración del Entorno](#configuración-del-entorno)
2. [Estructura del Código](#estructura-del-código)
3. [Compilación](#compilación)
4. [Testing](#testing)
5. [Debugging](#debugging)
6. [Contribución](#contribución)
7. [Convenciones de Código](#convenciones-de-código)

## 🔧 Configuración del Entorno

### Requisitos Previos

- **Rust**: 1.70 o superior
- **QEMU**: 6.0 o superior
- **Limine Bootloader**: v2
- **Python**: 3.8 o superior
- **Make**: 4.0 o superior

### Instalación en Linux

```bash
# Instalar Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Instalar QEMU
sudo apt install qemu-system-x86 qemu-utils

# Instalar Limine
git clone https://github.com/limine-bootloader/limine.git
cd limine
make
sudo make install

# Instalar herramientas de desarrollo
sudo apt install build-essential nasm python3
```

### Instalación en Windows

```powershell
# Instalar Rust
# Descargar desde https://rustup.rs/

# Instalar QEMU
# Descargar desde https://www.qemu.org/

# Instalar Python
# Descargar desde https://www.python.org/

# Instalar Make
# Usar WSL o MinGW
```

## 📁 Estructura del Código

### Directorio Principal

```
cronos_w-os/
├── bootloader/          # Bootloader
│   ├── main.rs
│   ├── memory.rs
│   ├── hardware.rs
│   ├── interrupts.rs
│   ├── ia_colmena_integration.rs
│   └── limine_boot.rs
├── kernel/             # Kernel
│   ├── lib.rs
│   ├── exokernel_graph.rs
│   ├── graph_memory.rs
│   ├── hardware_adaptation.rs
│   ├── universal_driver.rs
│   ├── scheduler.rs
│   ├── filesystem.rs
│   ├── networking.rs
│   ├── security.rs
│   ├── graphics.rs
│   ├── genesis.rs
│   ├── ia_colmena.rs
│   ├── crystal_ui.rs
│   ├── acpi.rs
│   ├── firmware_loader.rs
│   └── vga_buffer.rs
├── qemu/               # Scripts de QEMU
├── tests/              # Tests
├── Cargo.toml          # Configuración Rust
└── Makefile            # Configuración Make
```

### Convenciones de Nombres

- **Archivos**: snake_case (ej: `exokernel_graph.rs`)
- **Tipos**: PascalCase (ej: `ExokernelGraphSystem`)
- **Funciones**: snake_case (ej: `create_node`)
- **Constantes**: SCREAMING_SNAKE_CASE (ej: `BUFFER_HEIGHT`)
- **Módulos**: snake_case (ej: `exokernel_graph`)

## 🔨 Compilación

### Compilar el Kernel

```bash
make kernel
```

### Compilar el Bootloader

```bash
make bootloader
```

### Crear Imagen de Disco

```bash
make image
```

### Compilar Todo

```bash
make all
```

### Limpieza

```bash
make clean
```

## 🧪 Testing

### Ejecutar Todos los Tests

```bash
make test
```

### Tests Unitarios

```bash
cargo test --test unit
```

### Tests de Integración

```bash
cargo test --test integration
```

### Tests de Rendimiento

```bash
cargo test --test performance
```

### Tests Específicos

```bash
cargo test test_node_creation
```

## 🐛 Debugging

### Ejecutar en Modo Debug

```bash
make debug
```

### Ejecutar con GDB

```bash
./qemu/launch_script.sh -g
```

### Conectar GDB

```bash
gdb
(gdb) target remote localhost:1234
(gdb) break kernel_main
(gdb) continue
```

### Logging

```bash
# Habilitar logging en QEMU
./qemu/launch_script.sh -d

# Ver logs
cat qemu.log
```

## 🤝 Contribución

### Flujo de Trabajo

1. Fork el repositorio
2. Crea una rama para tu feature
3. Haz tus cambios
4. Escribe tests
5. Compila y ejecuta tests
6. Commit tus cambios
7. Push a tu rama
8. Abre un Pull Request

### Commit Messages

```
tipo: descripción corta

Descripción detallada del cambio.

Tipo puede ser:
- feat: nueva característica
- fix: corrección de bug
- docs: documentación
- style: formato
- refactor: refactorización
- test: tests
- chore: mantenimiento
```

### Code Review

- Revisa tu código antes de enviar
- Asegúrate de que los tests pasen
- Sigue las convenciones de código
- Documenta cambios importantes

## 📝 Convenciones de Código

### Rust

```rust
// Buen estilo
pub fn create_node(&mut self, name: String) -> u64 {
    let node_id = self.next_node_id;
    self.next_node_id += 1;
    
    let node = GraphNode {
        id: node_id,
        name,
        // ...
    };
    
    self.graph.add_node(node);
    node_id
}

// Mal estilo
pub fn CreateNode(&mut self, n:String)->u64{
    let i=self.next_node_id;
    self.next_node_id+=1;
    let node=GraphNode{id:i,name:n};
    self.graph.add_node(node);
    i
}
```

### Documentación

```rust
/// Crea un nuevo nodo en el grafo de recursos
///
/// # Argumentos
///
/// * `name` - Nombre del nodo
///
/// # Retorna
///
/// ID del nodo creado
///
/// # Ejemplos
///
/// ```
/// let node_id = system.create_node("CPU".to_string());
/// ```
pub fn create_node(&mut self, name: String) -> u64 {
    // ...
}
```

### Errores

```rust
// Usar Result para operaciones que pueden fallar
pub fn allocate_memory(&mut self, size: u64) -> Result<u64, MemoryError> {
    if size > MAX_SIZE {
        return Err(MemoryError::OutOfMemory);
    }
    // ...
}

// Usar Option para valores opcionales
pub fn get_node(&self, node_id: u64) -> Option<&GraphNode> {
    self.graph.get_node(node_id)
}
```

## 🚀 Despliegue

### Crear Release

```bash
# Compilar en modo release
cargo build --release

# Crear imagen de disco
make image

# Probar en QEMU
make run
```

### Crear ISO

```bash
# Crear ISO para distribución
make iso
```

## 📚 Recursos

### Documentación

- [Rust Book](https://doc.rust-lang.org/book/)
- [The Rustonomicon](https://doc.rust-lang.org/nomicon/)
- [Limine Documentation](https://github.com/limine-bootloader/limine)
- [OSDev Wiki](https://wiki.osdev.org/)

### Comunidad

- [Rust Forums](https://users.rust-lang.org/)
- [OSDev Forums](https://forum.osdev.org/)
- [CRONOS W-OS Discord](https://discord.gg/cronos)

## 🐛 Solución de Problemas

### Errores Comunes

**Error: `no_std` no soportado**
```bash
# Asegúrate de usar el target correcto
rustup target add x86_64-unknown-none
cargo build --target x86_64-unknown-none
```

**Error: QEMU no encontrado**
```bash
# Instala QEMU
sudo apt install qemu-system-x86
```

**Error: Bootloader no funciona**
```bash
# Reinstala Limine
cd limine
make clean
make
sudo make install
```

## 📈 Roadmap

### v2.0 (Actual)
- Exokernel con grafos
- Graph memory system
- Hardware adaptation
- Universal drivers
- Scheduler CFS
- Filesystem completo
- Network stack
- Security AEGIS
- Graphics LUMEN
- GENESIS auto-creation
- IA Colmena integration
- Crystal UI

### v2.1 (Próximo)
- Optimización de rendimiento
- Más drivers universales
- Mejor integración IA
- Más aplicaciones en Crystal UI

### v3.0 (Futuro)
- Soporte multiprocesador completo
- Soporte NUMA
- Encriptación cuántica real
- Más modelos de IA
