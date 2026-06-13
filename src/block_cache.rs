//! Block Cache Module
//! 
//! This module implements a block cache for filesystem operations.

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::format;

/// Entrada de caché de bloques
#[derive(Debug, Clone)]
pub struct CacheEntry {
    /// Número de bloque
    pub block_number: u64,
    /// Datos del bloque
    pub data: Vec<u8>,
    /// Sucio (modificado)
    pub dirty: bool,
    /// Último acceso
    pub last_access: u64,
    /// Referencias activas
    pub ref_count: u32,
}

impl CacheEntry {
    /// Crear nueva entrada de caché
    pub fn new(block_number: u64, data: Vec<u8>) -> Self {
        Self {
            block_number,
            data,
            dirty: false,
            last_access: 0,
            ref_count: 0,
        }
    }

    /// Marcar como sucio
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Marcar como limpio
    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    /// Incrementar referencia
    pub fn inc_ref(&mut self) {
        self.ref_count += 1;
    }

    /// Decrementar referencia
    pub fn dec_ref(&mut self) {
        if self.ref_count > 0 {
            self.ref_count -= 1;
        }
    }

    /// Verificar si está en uso
    pub fn is_in_use(&self) -> bool {
        self.ref_count > 0
    }
}

/// Estadísticas de caché
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Hits de caché
    pub hits: u64,
    /// Misses de caché
    pub misses: u64,
    /// Bloques sucios
    pub dirty_blocks: u64,
    /// Bloques escritos
    pub writes: u64,
    /// Bloques leídos
    pub reads: u64,
}

impl CacheStats {
    /// Crear nuevas estadísticas
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            dirty_blocks: 0,
            writes: 0,
            reads: 0,
        }
    }

    /// Calcular ratio de hit
    pub fn hit_ratio(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            (self.hits as f64) / (total as f64)
        }
    }
}

impl Default for CacheStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Caché de bloques
pub struct BlockCache {
    /// Entradas de caché
    pub entries: Vec<CacheEntry>,
    /// Tamaño máximo de caché
    pub max_size: usize,
    /// Tamaño de bloque
    pub block_size: usize,
    /// Estadísticas
    pub stats: CacheStats,
    /// Tiempo actual
    pub current_time: u64,
}

impl BlockCache {
    /// Crear nueva caché de bloques
    pub fn new(max_size: usize, block_size: usize) -> Self {
        Self {
            entries: Vec::new(),
            max_size,
            block_size,
            stats: CacheStats::new(),
            current_time: 0,
        }
    }

    /// Leer bloque desde caché
    pub fn read(&mut self, block_number: u64) -> Option<Vec<u8>> {
        self.current_time += 1;
        
        // Buscar en caché
        for entry in &mut self.entries {
            if entry.block_number == block_number {
                entry.last_access = self.current_time;
                entry.inc_ref();
                self.stats.hits += 1;
                self.stats.reads += 1;
                return Some(entry.data.clone());
            }
        }
        
        // Cache miss
        self.stats.misses += 1;
        self.stats.reads += 1;
        None
    }

    /// Escribir bloque a caché
    pub fn write(&mut self, block_number: u64, data: Vec<u8>) -> Result<(), String> {
        if data.len() != self.block_size {
            return Err(format!("Data size {} does not match block size {}", data.len(), self.block_size));
        }

        self.current_time += 1;
        
        // Buscar si ya existe en caché
        for entry in &mut self.entries {
            if entry.block_number == block_number {
                entry.data = data;
                entry.mark_dirty();
                entry.last_access = self.current_time;
                self.stats.writes += 1;
                return Ok(());
            }
        }
        
        // Si no existe, agregar nuevo
        if self.entries.len() >= self.max_size {
            self.evict();
        }
        
        let mut entry = CacheEntry::new(block_number, data);
        entry.mark_dirty();
        entry.last_access = self.current_time;
        self.entries.push(entry);
        self.stats.writes += 1;
        
        Ok(())
    }

    /// Evictar entrada usando LRU
    fn evict(&mut self) {
        // Encontrar entrada menos usada recientemente que no esté en uso
        let mut lru_index = 0;
        let mut lru_time = u64::MAX;
        
        for (i, entry) in self.entries.iter().enumerate() {
            if !entry.is_in_use() && entry.last_access < lru_time {
                lru_index = i;
                lru_time = entry.last_access;
            }
        }
        
        // Si la entrada está sucia, en un sistema real se escribiría a disco
        if self.entries[lru_index].dirty {
            self.stats.dirty_blocks -= 1;
        }
        
        self.entries.remove(lru_index);
    }

    /// Liberar bloque
    pub fn release(&mut self, block_number: u64) {
        for entry in &mut self.entries {
            if entry.block_number == block_number {
                entry.dec_ref();
                break;
            }
        }
    }

    /// Sincronizar bloques sucios
    pub fn sync(&mut self) -> Result<(), String> {
        for entry in &mut self.entries {
            if entry.dirty {
                // En un sistema real, esto escribiría el bloque a disco
                entry.mark_clean();
                self.stats.dirty_blocks -= 1;
            }
        }
        Ok(())
    }

    /// Invalidar caché
    pub fn invalidate(&mut self) {
        self.entries.clear();
        self.stats = CacheStats::new();
    }

    /// Invalidar bloque específico
    pub fn invalidate_block(&mut self, block_number: u64) {
        self.entries.retain(|e| e.block_number != block_number);
    }

    /// Obtener número de bloques sucios
    pub fn dirty_count(&self) -> usize {
        self.entries.iter().filter(|e| e.dirty).count()
    }

    /// Obtener tamaño actual de caché
    pub fn current_size(&self) -> usize {
        self.entries.len()
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("Block Cache Status\n");
        report.push_str("==================\n\n");
        
        report.push_str(&format!("Max Size: {} blocks\n", self.max_size));
        report.push_str(&format!("Current Size: {} blocks\n", self.current_size()));
        report.push_str(&format!("Block Size: {} bytes\n", self.block_size));
        report.push_str(&format!("Dirty Blocks: {}\n", self.dirty_count()));
        report.push_str(&format!("Cache Hits: {}\n", self.stats.hits));
        report.push_str(&format!("Cache Misses: {}\n", self.stats.misses));
        report.push_str(&format!("Hit Ratio: {:.2}%\n", self.stats.hit_ratio() * 100.0));
        report.push_str(&format!("Reads: {}\n", self.stats.reads));
        report.push_str(&format!("Writes: {}\n", self.stats.writes));
        
        report
    }
}

impl Default for BlockCache {
    fn default() -> Self {
        Self::new(1024, 4096)
    }
}
