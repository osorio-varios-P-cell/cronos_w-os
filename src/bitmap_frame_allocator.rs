//! BitmapFrameAllocator — Asignador de frames físicos O(1) por palabra
//!
//! Sustituye la búsqueda lineal de `BootInfoFrameAllocator` por un bitmap
//! donde cada bit representa un frame de 4 KiB. La búsqueda del primer bit
//! libre usa la instrucción BSF (Bit Scan Forward) vía `u64::trailing_zeros`.
//!
//! Mejoras respecto al original:
//!   • Asignación O(n/64) en vez de O(n) (n = número de frames)
//!   • `free_frame()` funcional (el original era un stub vacío)
//!   • Soporte de zonas: DMA (<16 MiB), Normal (16 MiB–4 GiB), High (>4 GiB)
//!   • Estadísticas de uso para el monitor de AEGIS

extern crate alloc;

use alloc::vec;

use x86_64::{
    structures::paging::{FrameAllocator, PhysFrame, Size4KiB},
    PhysAddr,
};

/// Tamaño de un frame en bytes
const FRAME_SIZE: u64 = 4096;

/// Número máximo de frames soportados (4 GiB / 4 KiB = 1M frames)
const MAX_FRAMES: usize = 1024 * 1024;

/// Número de palabras u64 necesarias para el bitmap
const BITMAP_WORDS: usize = MAX_FRAMES / 64;

/// Zona de memoria
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemZone {
    /// DMA: 0 – 16 MiB (compatible con ISA DMA)
    Dma,
    /// Normal: 16 MiB – 4 GiB
    Normal,
    /// High: > 4 GiB (requiere PAE/x86_64)
    High,
}

impl MemZone {
    fn of(phys_addr: u64) -> Self {
        if phys_addr < 16 * 1024 * 1024 { Self::Dma }
        else if phys_addr < 4 * 1024 * 1024 * 1024 { Self::Normal }
        else { Self::High }
    }
}

/// Asignador de frames basado en bitmap dinámico.
///
/// `bitmap[i]` tiene el bit `j` a 1 si el frame `i*64+j` está **libre**.
#[derive(Debug)]
pub struct BitmapFrameAllocator {
    bitmap: alloc::vec::Vec<u64>,
    total_frames:  usize,
    free_frames:   usize,
    base_addr:     u64,   // dirección física del primer frame gestionado
}

impl BitmapFrameAllocator {
    /// Crea un asignador vacío (todos los frames marcados como usados).
    pub fn new() -> Self {
        Self {
            bitmap: alloc::vec![0u64; BITMAP_WORDS],
            total_frames: 0,
            free_frames: 0,
            base_addr: 0,
        }
    }

    /// Inicializa el asignador con las regiones de memoria disponibles.
    ///
    /// # Safety
    /// Las regiones deben representar memoria física real y accesible.
    pub unsafe fn init(&mut self, base_addr: u64, regions: &[(u64, u64)]) {
        self.base_addr = base_addr;
        // Marcar todas las regiones disponibles como libres
        for &(start, end) in regions {
            self.mark_range_free(start, end);
        }
    }

    /// Marca un rango [start, end) de direcciones físicas como libre.
    fn mark_range_free(&mut self, start: u64, end: u64) {
        let first_frame = ((start.saturating_sub(self.base_addr)) / FRAME_SIZE) as usize;
        let last_frame  = ((end.saturating_sub(self.base_addr)) / FRAME_SIZE) as usize;
        for frame_idx in first_frame..last_frame.min(MAX_FRAMES) {
            self.set_free(frame_idx);
            self.total_frames += 1;
            self.free_frames  += 1;
        }
    }

    /// Marca el frame `idx` como libre.
    #[inline]
    fn set_free(&mut self, idx: usize) {
        if idx < MAX_FRAMES {
            self.bitmap[idx / 64] |= 1u64 << (idx % 64);
        }
    }

    /// Marca el frame `idx` como usado.
    #[inline]
    fn set_used(&mut self, idx: usize) {
        if idx < MAX_FRAMES {
            self.bitmap[idx / 64] &= !(1u64 << (idx % 64));
        }
    }

    /// Busca el primer frame libre en una zona específica.
    fn find_free_in_zone(&self, zone: MemZone) -> Option<usize> {
        let (min_frame, max_frame) = match zone {
            MemZone::Dma    => (0, (16 * 1024 * 1024 / FRAME_SIZE) as usize),
            MemZone::Normal => ((16 * 1024 * 1024 / FRAME_SIZE) as usize,
                                (4u64 * 1024 * 1024 * 1024 / FRAME_SIZE) as usize),
            MemZone::High   => ((4u64 * 1024 * 1024 * 1024 / FRAME_SIZE) as usize, MAX_FRAMES),
        };

        let min_word = min_frame / 64;
        let max_word = (max_frame / 64).min(BITMAP_WORDS);

        for word_idx in min_word..max_word {
            let word = self.bitmap[word_idx];
            if word != 0 {
                // BSF: bit scan forward — O(1) en hardware
                let bit = word.trailing_zeros() as usize;
                let frame_idx = word_idx * 64 + bit;
                if frame_idx >= min_frame && frame_idx < max_frame {
                    return Some(frame_idx);
                }
            }
        }
        None
    }

    /// Asigna un frame de la zona especificada.
    pub fn alloc_frame_in_zone(&mut self, zone: MemZone) -> Option<PhysFrame> {
        let idx = self.find_free_in_zone(zone)?;
        self.set_used(idx);
        self.free_frames = self.free_frames.saturating_sub(1);
        let addr = self.base_addr + idx as u64 * FRAME_SIZE;
        // SAFETY: la dirección viene de una región marcada como libre por el bootloader.
        Some(unsafe { PhysFrame::containing_address(PhysAddr::new(addr)) })
    }

    /// Libera un frame físico. El original tenía esta función vacía — ahora funciona.
    pub fn free_frame(&mut self, frame: PhysFrame) {
        let addr = frame.start_address().as_u64();
        if addr < self.base_addr { return; }
        let idx = ((addr - self.base_addr) / FRAME_SIZE) as usize;
        if idx >= MAX_FRAMES { return; }
        // Verificar que no estemos liberando un frame ya libre (double-free)
        if self.bitmap[idx / 64] & (1u64 << (idx % 64)) != 0 {
            // Ya estaba libre: ignorar silenciosamente (o loggear con kerror!)
            return;
        }
        self.set_free(idx);
        self.free_frames += 1;
    }

    /// Estadísticas de memoria.
    pub fn stats(&self) -> AllocStats {
        AllocStats {
            total_frames: self.total_frames,
            free_frames:  self.free_frames,
            used_frames:  self.total_frames.saturating_sub(self.free_frames),
            total_mb:     self.total_frames * 4 / 1024,
            free_mb:      self.free_frames  * 4 / 1024,
        }
    }
}

/// Implementación de `FrameAllocator` para integración con x86_64 crate.
///
/// Por defecto asigna de la zona Normal; usar `alloc_frame_in_zone` para DMA.
unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        // Intentar Normal primero, luego DMA, luego High
        self.alloc_frame_in_zone(MemZone::Normal)
            .or_else(|| self.alloc_frame_in_zone(MemZone::Dma))
            .or_else(|| self.alloc_frame_in_zone(MemZone::High))
    }
}

#[derive(Debug, Clone)]
pub struct AllocStats {
    pub total_frames: usize,
    pub free_frames:  usize,
    pub used_frames:  usize,
    pub total_mb:     usize,
    pub free_mb:      usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alloc_free() {
        let mut alloc = BitmapFrameAllocator::new();
        // Simular 1 MiB de memoria normal (16 MiB – 17 MiB para caer en Normal)
        unsafe {
            alloc.base_addr = 16 * 1024 * 1024;
            alloc.mark_range_free(16 * 1024 * 1024, 17 * 1024 * 1024);
        }
        let frame1 = alloc.alloc_frame_in_zone(MemZone::Normal).expect("debe haber frame libre");
        let frame2 = alloc.alloc_frame_in_zone(MemZone::Normal).expect("debe haber frame libre");
        assert_ne!(frame1.start_address(), frame2.start_address());
        // Liberar frame1 y volverlo a asignar
        let addr1 = frame1.start_address();
        alloc.free_frame(frame1);
        let frame1b = alloc.alloc_frame_in_zone(MemZone::Normal).expect("frame1 recién liberado");
        assert_eq!(frame1b.start_address(), addr1);
    }

    #[test]
    fn test_no_double_free() {
        let mut alloc = BitmapFrameAllocator::new();
        unsafe {
            alloc.base_addr = 16 * 1024 * 1024;
            alloc.mark_range_free(16 * 1024 * 1024, 17 * 1024 * 1024);
        }
        let initial_free = alloc.stats().free_frames;
        let frame = alloc.alloc_frame_in_zone(MemZone::Normal).unwrap();
        alloc.free_frame(frame);
        // Double-free no debe cambiar el contador
        alloc.free_frame(frame);
        assert_eq!(alloc.stats().free_frames, initial_free);
    }

    #[test]
    fn test_zone_dma() {
        let mut alloc = BitmapFrameAllocator::new();
        unsafe {
            alloc.base_addr = 0;
            alloc.mark_range_free(0, 16 * 1024 * 1024); // DMA zone
        }
        let frame = alloc.alloc_frame_in_zone(MemZone::Dma);
        assert!(frame.is_some());
        // No debe haber nada en Normal
        assert!(alloc.alloc_frame_in_zone(MemZone::Normal).is_none());
    }
}
