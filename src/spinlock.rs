//! Spinlock seguro para SMP en entorno no_std
//!
//! Usa la instrucción PAUSE de x86_64 para reducir el consumo de energía
//! durante la espera activa, respetando la coherencia de cache.
//!
//! Diferencia con spin::Mutex: este spinlock es audit-friendly (se puede
//! integrar con AEGIS) y tiene métricas de contención.

use core::cell::UnsafeCell;
use core::hint;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

/// Contador global de contenciones de spinlock (para métricas de AEGIS)
static CONTENTION_COUNT: core::sync::atomic::AtomicU64 = core::sync::atomic::AtomicU64::new(0);

/// Un Mutex basado en spinlock para entornos no_std con soporte SMP.
///
/// Garantías:
/// - Exclusión mutua correcta en múltiples núcleos.
/// - No requiere asignación dinámica.
/// - Integrable con el sistema de capabilities de CRONOS.
#[derive(Debug)]
pub struct SpinMutex<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: El acceso mutuo exclusivo está garantizado por el spinlock.
unsafe impl<T: Send> Send for SpinMutex<T> {}
unsafe impl<T: Send> Sync for SpinMutex<T> {}

impl<T> SpinMutex<T> {
    /// Crea un nuevo SpinMutex con el valor dado.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(value),
        }
    }

    /// Adquiere el lock, girando (spin) hasta conseguirlo.
    ///
    /// Usa backoff exponencial con PAUSE para reducir contención de bus.
    pub fn lock(&self) -> SpinGuard<'_, T> {
        let mut spin_count = 0u32;
        while self
            .locked
            .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            // Backoff exponencial: espera más tiempo a mayor contención
            let backoff = 1u32 << spin_count.min(6);
            for _ in 0..backoff {
                hint::spin_loop(); // emite PAUSE en x86, WFE en ARM
            }
            if spin_count < 6 {
                spin_count += 1;
            } else {
                // Registrar contención prolongada para AEGIS
                CONTENTION_COUNT.fetch_add(1, Ordering::Relaxed);
            }
        }
        SpinGuard { mutex: self }
    }

    /// Intenta adquirir el lock sin bloquear.
    ///
    /// Devuelve `None` si el lock ya está tomado.
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        if self
            .locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
        {
            Some(SpinGuard { mutex: self })
        } else {
            None
        }
    }

    /// Devuelve el número total de contenciones prolongadas registradas.
    /// Útil para que AEGIS detecte posibles deadlocks o carga excesiva.
    pub fn global_contention_count() -> u64 {
        CONTENTION_COUNT.load(Ordering::Relaxed)
    }
}

/// Guard RAII que libera el lock al salir de scope.
pub struct SpinGuard<'a, T> {
    mutex: &'a SpinMutex<T>,
}

impl<T> Deref for SpinGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: somos el único propietario del lock.
        unsafe { &*self.mutex.data.get() }
    }
}

impl<T> DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: somos el único propietario del lock.
        unsafe { &mut *self.mutex.data.get() }
    }
}

impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(false, Ordering::Release);
    }
}

/// RwSpinLock: permite múltiples lectores simultáneos o un escritor exclusivo.
///
/// El campo `state` codifica:
///   - 0          → libre
///   - 0x8000_0000 → escritor activo
///   - N (1..N)   → N lectores activos
pub struct RwSpinLock<T> {
    state: core::sync::atomic::AtomicU32,
    data: UnsafeCell<T>,
}

const WRITER_FLAG: u32 = 0x8000_0000;

unsafe impl<T: Send> Send for RwSpinLock<T> {}
unsafe impl<T: Send + Sync> Sync for RwSpinLock<T> {}

impl<T> RwSpinLock<T> {
    pub const fn new(value: T) -> Self {
        Self {
            state: core::sync::atomic::AtomicU32::new(0),
            data: UnsafeCell::new(value),
        }
    }

    /// Adquiere acceso de lectura (compartido).
    pub fn read(&self) -> ReadGuard<'_, T> {
        loop {
            let s = self.state.load(Ordering::Relaxed);
            // Fallar si hay escritor activo o si contador desborda
            if s & WRITER_FLAG == 0 && s < WRITER_FLAG - 1 {
                if self
                    .state
                    .compare_exchange_weak(s, s + 1, Ordering::Acquire, Ordering::Relaxed)
                    .is_ok()
                {
                    return ReadGuard { lock: self };
                }
            }
            hint::spin_loop();
        }
    }

    /// Adquiere acceso de escritura (exclusivo).
    pub fn write(&self) -> WriteGuard<'_, T> {
        loop {
            if self
                .state
                .compare_exchange_weak(0, WRITER_FLAG, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return WriteGuard { lock: self };
            }
            hint::spin_loop();
        }
    }
}

pub struct ReadGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for ReadGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> Drop for ReadGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.fetch_sub(1, Ordering::Release);
    }
}

pub struct WriteGuard<'a, T> {
    lock: &'a RwSpinLock<T>,
}

impl<T> Deref for WriteGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for WriteGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for WriteGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.state.store(0, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinmutex_basic() {
        let m = SpinMutex::new(0u32);
        {
            let mut g = m.lock();
            *g = 42;
        }
        assert_eq!(*m.lock(), 42);
    }

    #[test]
    fn test_try_lock() {
        let m = SpinMutex::new(0u32);
        let g = m.lock();
        // No se puede tomar el lock dos veces
        assert!(m.try_lock().is_none());
        drop(g);
        assert!(m.try_lock().is_some());
    }

    #[test]
    fn test_rwlock_multiple_readers() {
        let rw = RwSpinLock::new(99u32);
        let r1 = rw.read();
        let r2 = rw.read();
        assert_eq!(*r1, 99);
        assert_eq!(*r2, 99);
    }
}
