//! Cache System de CRONOS original adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el sistema de caché multi-nivel de CRONOS original,
//! adaptado al sistema de capabilities y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::BTreeMap;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Política de reemplazo de caché
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CachePolicy {
    LRU,       // Least Recently Used
    LFU,       // Least Frequently Used
    FIFO,      // First In First Out
    Random,    // Random replacement
}

/// Nivel de caché
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CacheLevel {
    L1,        // Nivel 1 - Memoria caché rápida
    L2,        // Nivel 2 - Memoria caché secundaria
    L3,        // Nivel 3 - Memoria caché terciaria
    Disk,      // Caché en disco
}

/// Entrada de caché
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub size: u64,
    pub access_count: u32,
    pub last_access: u64,
    pub created_at: u64,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl CacheEntry {
    pub fn new(key: String, value: Vec<u8>) -> Self {
        let size = value.len() as u64;
        Self {
            key,
            value,
            size,
            access_count: 0,
            last_access: 0,
            created_at: 0,
            graph_node_id: None,
        }
    }

    pub fn touch(&mut self) {
        self.access_count += 1;
        self.last_access = 0; // Simulación de timestamp actual
    }
}

/// Estadísticas de caché
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub size: u64,
    pub entries: u32,
}

impl CacheStats {
    pub fn new() -> Self {
        Self {
            hits: 0,
            misses: 0,
            evictions: 0,
            size: 0,
            entries: 0,
        }
    }

    pub fn hit_rate(&self) -> f64 {
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

/// Configuración de caché
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_size: u64,
    pub max_entries: u32,
    pub policy: CachePolicy,
    pub level: CacheLevel,
    pub ttl: u64, // Time to live en segundos
}

impl CacheConfig {
    pub fn new(level: CacheLevel) -> Self {
        let (max_size, max_entries) = match level {
            CacheLevel::L1 => (256 * 1024, 1024),          // 256KB, 1024 entradas
            CacheLevel::L2 => (2 * 1024 * 1024, 4096),    // 2MB, 4096 entradas
            CacheLevel::L3 => (16 * 1024 * 1024, 16384),   // 16MB, 16384 entradas
            CacheLevel::Disk => (1024 * 1024 * 1024, 65536), // 1GB, 65536 entradas
        };

        Self {
            max_size,
            max_entries,
            policy: CachePolicy::LRU,
            level,
            ttl: 3600, // 1 hora por defecto
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self::new(CacheLevel::L2)
    }
}

/// Caché
#[derive(Debug, Clone)]
pub struct CronosCache {
    pub config: CacheConfig,
    pub entries: BTreeMap<String, CacheEntry>,
    pub stats: CacheStats,
    pub access_order: Vec<String>, // Para LRU
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosCache {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            entries: BTreeMap::new(),
            stats: CacheStats::new(),
            access_order: Vec::new(),
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Obtener valor de caché
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.entries.get_mut(key) {
            entry.touch();
            
            // Actualizar orden de acceso para LRU
            if self.config.policy == CachePolicy::LRU {
                self.access_order.retain(|k| k != key);
                self.access_order.push(String::from(key));
            }

            self.stats.hits += 1;
            Some(entry.value.clone())
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insertar valor en caché
    pub fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), String> {
        let mut entry = CacheEntry::new(key.clone(), value);

        // Registrar la entrada como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::MemoryRegion;
            let node_name = format!("cache_entry_{}", key);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            entry.graph_node_id = node_id;
        }
        
        // Verificar si hay espacio
        if self.entries.len() >= self.config.max_entries as usize {
            self.evict()?;
        }

        if self.stats.size + entry.size > self.config.max_size {
            self.evict()?;
        }

        // Insertar entrada
        self.stats.size += entry.size;
        self.stats.entries = self.entries.len() as u32;
        self.entries.insert(key.clone(), entry);

        // Actualizar orden de acceso para LRU
        if self.config.policy == CachePolicy::LRU {
            self.access_order.push(key);
        }

        Ok(())
    }

    /// Remover entrada de caché
    pub fn remove(&mut self, key: &str) -> Option<CacheEntry> {
        if let Some(entry) = self.entries.remove(key) {
            self.stats.size -= entry.size;
            self.stats.entries = self.entries.len() as u32;
            
            // Actualizar orden de acceso para LRU
            if self.config.policy == CachePolicy::LRU {
                self.access_order.retain(|k| k != key);
            }

            Some(entry)
        } else {
            None
        }
    }

    /// Verificar si existe una clave
    pub fn contains(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    /// Limpiar caché
    pub fn clear(&mut self) {
        self.entries.clear();
        self.access_order.clear();
        self.stats = CacheStats::new();
    }

    /// Obtener tamaño actual
    pub fn size(&self) -> u64 {
        self.stats.size
    }

    /// Obtener número de entradas
    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    /// Evict (reemplazar) entrada según política
    fn evict(&mut self) -> Result<(), String> {
        if self.entries.is_empty() {
            return Err(String::from("Cache empty"));
        }

        let key_to_remove = match self.config.policy {
            CachePolicy::LRU => {
                // Eliminar el menos recientemente usado
                self.access_order.first().cloned()
            }
            CachePolicy::LFU => {
                // Eliminar el menos frecuentemente usado
                self.entries.iter()
                    .min_by_key(|(_, e)| e.access_count)
                    .map(|(k, _)| k.clone())
            }
            CachePolicy::FIFO => {
                // Eliminar el primero en entrar
                self.access_order.first().cloned()
            }
            CachePolicy::Random => {
                // Eliminar aleatoriamente
                let keys: Vec<_> = self.entries.keys().cloned().collect();
                if keys.is_empty() {
                    None
                } else {
                    Some(keys[keys.len() / 2].clone())
                }
            }
        };

        if let Some(key) = key_to_remove {
            if let Some(entry) = self.remove(&key) {
                self.stats.evictions += 1;
            }
        }

        Ok(())
    }

    /// Obtener estadísticas
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }
}

impl Default for CronosCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

/// Sistema de caché multi-nivel
#[derive(Debug, Clone)]
pub struct CronosCacheSystem {
    pub caches: BTreeMap<CacheLevel, CronosCache>,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosCacheSystem {
    pub fn new() -> Self {
        let mut caches = BTreeMap::new();
        caches.insert(CacheLevel::L1, CronosCache::new(CacheConfig::new(CacheLevel::L1)));
        caches.insert(CacheLevel::L2, CronosCache::new(CacheConfig::new(CacheLevel::L2)));
        caches.insert(CacheLevel::L3, CronosCache::new(CacheConfig::new(CacheLevel::L3)));
        caches.insert(CacheLevel::Disk, CronosCache::new(CacheConfig::new(CacheLevel::Disk)));

        Self {
            caches,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel.clone()));
        for cache in self.caches.values_mut() {
            cache.set_graph_kernel(graph_kernel.clone());
        }
    }

    /// Obtener valor (busca en todos los niveles)
    pub fn get(&mut self, key: &str) -> Option<Vec<u8>> {
        // Buscar primero en L1, luego L2, L3, Disk
        for level in [CacheLevel::L1, CacheLevel::L2, CacheLevel::L3, CacheLevel::Disk] {
            if let Some(cache) = self.caches.get_mut(&level) {
                if let Some(value) = cache.get(key) {
                    // Promover a niveles superiores
                    self.promote(key, &value, level);
                    return Some(value);
                }
            }
        }
        None
    }

    /// Insertar valor (en todos los niveles)
    pub fn put(&mut self, key: String, value: Vec<u8>) -> Result<(), String> {
        for cache in self.caches.values_mut() {
            let _ = cache.put(key.clone(), value.clone());
        }
        Ok(())
    }

    /// Promover valor a niveles superiores
    fn promote(&mut self, key: &str, value: &[u8], current_level: CacheLevel) {
        match current_level {
            CacheLevel::L1 => {} // Ya está en el nivel más alto
            CacheLevel::L2 => {
                if let Some(cache) = self.caches.get_mut(&CacheLevel::L1) {
                    let _ = cache.put(String::from(key), value.to_vec());
                }
            }
            CacheLevel::L3 => {
                if let Some(cache) = self.caches.get_mut(&CacheLevel::L2) {
                    let _ = cache.put(String::from(key), value.to_vec());
                }
                if let Some(cache) = self.caches.get_mut(&CacheLevel::L1) {
                    let _ = cache.put(String::from(key), value.to_vec());
                }
            }
            CacheLevel::Disk => {
                if let Some(cache) = self.caches.get_mut(&CacheLevel::L3) {
                    let _ = cache.put(String::from(key), value.to_vec());
                }
                if let Some(cache) = self.caches.get_mut(&CacheLevel::L2) {
                    let _ = cache.put(String::from(key), value.to_vec());
                }
                if let Some(cache) = self.caches.get_mut(&CacheLevel::L1) {
                    let _ = cache.put(String::from(key), value.to_vec());
                }
            }
        }
    }

    /// Obtener caché de un nivel específico
    pub fn get_cache(&mut self, level: CacheLevel) -> Option<&mut CronosCache> {
        self.caches.get_mut(&level)
    }

    /// Obtener estadísticas combinadas
    pub fn combined_stats(&self) -> CacheSystemStats {
        let mut total_hits = 0;
        let mut total_misses = 0;
        let mut total_evictions = 0;
        let mut total_size = 0;
        let mut total_entries = 0;

        for cache in self.caches.values() {
            total_hits += cache.stats.hits;
            total_misses += cache.stats.misses;
            total_evictions += cache.stats.evictions;
            total_size += cache.stats.size;
            total_entries += cache.stats.entries;
        }

        CacheSystemStats {
            total_hits,
            total_misses,
            total_evictions,
            total_size,
            total_entries,
            hit_rate: if total_hits + total_misses > 0 {
                (total_hits as f64) / ((total_hits + total_misses) as f64)
            } else {
                0.0
            },
        }
    }
}

impl Default for CronosCacheSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Estadísticas del sistema de caché
#[derive(Debug, Clone)]
pub struct CacheSystemStats {
    pub total_hits: u64,
    pub total_misses: u64,
    pub total_evictions: u64,
    pub total_size: u64,
    pub total_entries: u32,
    pub hit_rate: f64,
}

/// Errores de caché
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    CacheEmpty,
    KeyNotFound,
    OutOfSpace,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CacheError::CacheEmpty => write!(f, "Cache empty"),
            CacheError::KeyNotFound => write!(f, "Key not found"),
            CacheError::OutOfSpace => write!(f, "Out of space"),
        }
    }
}
