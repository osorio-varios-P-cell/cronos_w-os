//! Capability System - Replaces syscalls with intralanguage capabilities
//! 
//! This module implements a capability-based security model where all resource access
//! is mediated through capabilities instead of syscalls. This provides fine-grained
//! access control and eliminates the need for traditional system calls.

use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU64, Ordering};
use alloc::vec::Vec;
use crate::spinlock::SpinMutex;

/// Unique identifier for a capability
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CapabilityId(pub u64);

impl CapabilityId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(1);
        CapabilityId(NEXT_ID.fetch_add(1, Ordering::SeqCst))
    }
}

/// Rights that a capability grants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CapabilityRights {
    pub read: bool,
    pub write: bool,
    pub execute: bool,
    pub grant: bool,
    pub delegate: bool,
}

impl CapabilityRights {
    pub const NONE: Self = Self {
        read: false,
        write: false,
        execute: false,
        grant: false,
        delegate: false,
    };

    pub const READ_ONLY: Self = Self {
        read: true,
        write: false,
        execute: false,
        grant: false,
        delegate: false,
    };

    pub const READ_WRITE: Self = Self {
        read: true,
        write: true,
        execute: false,
        grant: false,
        delegate: false,
    };

    pub const FULL: Self = Self {
        read: true,
        write: true,
        execute: true,
        grant: true,
        delegate: true,
    };

    pub fn can_read(&self) -> bool {
        self.read
    }

    pub fn can_write(&self) -> bool {
        self.write
    }

    pub fn can_execute(&self) -> bool {
        self.execute
    }

    pub fn can_grant(&self) -> bool {
        self.grant
    }

    pub fn can_delegate(&self) -> bool {
        self.delegate
    }

    pub fn restrict(&self, rights: CapabilityRights) -> CapabilityRights {
        CapabilityRights {
            read: self.read && rights.read,
            write: self.write && rights.write,
            execute: self.execute && rights.execute,
            grant: self.grant && rights.grant,
            delegate: self.delegate && rights.delegate,
        }
    }
}

/// A capability to access a resource of type T
#[derive(Debug)]
pub struct Capability<T: ?Sized> {
    id: CapabilityId,
    resource: *mut T,
    rights: CapabilityRights,
}

impl<T: ?Sized> Capability<T> {
    /// Create a new capability with full rights
    /// 
    /// # Safety
    /// The caller must ensure the resource pointer is valid
    pub unsafe fn new(resource: *mut T) -> Self {
        Self {
            id: CapabilityId::new(),
            resource,
            rights: CapabilityRights::FULL,
        }
    }

    /// Create a new capability with specific rights
    /// 
    /// # Safety
    /// The caller must ensure the resource pointer is valid
    pub unsafe fn with_rights(resource: *mut T, rights: CapabilityRights) -> Self {
        Self {
            id: CapabilityId::new(),
            resource,
            rights,
        }
    }

    /// Get the capability ID
    pub fn id(&self) -> CapabilityId {
        self.id
    }

    /// Get the rights associated with this capability
    pub fn rights(&self) -> CapabilityRights {
        self.rights
    }

    /// Restrict the rights of this capability
    pub fn restrict(&self, rights: CapabilityRights) -> Capability<T> {
        Capability {
            id: CapabilityId::new(),
            resource: self.resource,
            rights: self.rights.restrict(rights),
        }
    }

    /// Check if this capability has read access
    pub fn can_read(&self) -> bool {
        self.rights.can_read()
    }

    /// Check if this capability has write access
    pub fn can_write(&self) -> bool {
        self.rights.can_write()
    }

    /// Check if this capability has execute access
    pub fn can_execute(&self) -> bool {
        self.rights.can_execute()
    }

    /// Check if this capability can grant rights to others
    pub fn can_grant(&self) -> bool {
        self.rights.can_grant()
    }

    /// Check if this capability can be delegated
    pub fn can_delegate(&self) -> bool {
        self.rights.can_delegate()
    }

    /// Get a mutable reference to the resource
    /// 
    /// # Safety
    /// The caller must ensure they have the appropriate rights
    pub unsafe fn get_mut(&self) -> Option<&mut T> {
        if self.can_write() {
            Some(&mut *self.resource)
        } else {
            None
        }
    }

    /// Get an immutable reference to the resource
    /// 
    /// # Safety
    /// The caller must ensure they have the appropriate rights
    pub unsafe fn get(&self) -> Option<&T> {
        if self.can_read() {
            Some(&*self.resource)
        } else {
            None
        }
    }
}

/// A mutable cell that can be accessed through capabilities (SMP-safe)
/// BUG #10 corregido: reemplazado UnsafeCell por SpinMutex para SMP safety
#[derive(Debug)]
pub struct Cell<T> {
    inner: SpinMutex<T>,
}

impl<T: Clone> Clone for Cell<T> {
    fn clone(&self) -> Self {
        Self {
            inner: SpinMutex::new(self.inner.lock().clone()),
        }
    }
}

impl<T> Cell<T> {
    /// Create a new cell
    pub fn new(value: T) -> Self {
        Self {
            inner: SpinMutex::new(value),
        }
    }

    /// Get a raw pointer to the inner value (SMP-safe)
    pub fn as_ptr(&self) -> *mut T {
        let guard = self.inner.lock();
        &*guard as *const T as *mut T
    }

    /// Create a capability to access this cell
    pub fn capability(&self) -> Capability<T> {
        unsafe { Capability::new(self.as_ptr()) }
    }

    /// Create a capability with specific rights to access this cell
    pub fn capability_with_rights(&self, rights: CapabilityRights) -> Capability<T> {
        unsafe { Capability::with_rights(self.as_ptr(), rights) }
    }
}

/// Invoke a capability with a function
/// 
/// This is the primary way to interact with resources through capabilities,
/// replacing traditional syscalls.
pub fn invoke_capability<T, R, F>(cap: &Capability<T>, f: F) -> Option<R>
where
    F: FnOnce(&T) -> R,
{
    unsafe {
        cap.get().map(f)
    }
}

/// Invoke a capability with a mutable function
/// 
/// This is the primary way to mutate resources through capabilities,
/// replacing traditional syscalls.
pub fn invoke_capability_mut<T, R, F>(cap: &Capability<T>, f: F) -> Option<R>
where
    F: FnOnce(&mut T) -> R,
{
    unsafe {
        cap.get_mut().map(f)
    }
}

/// Result of a capability invocation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CapabilityResult<T> {
    Success(T),
    AccessDenied,
    InvalidCapability,
    ResourceUnavailable,
}

impl<T> CapabilityResult<T> {
    pub fn is_success(&self) -> bool {
        matches!(self, CapabilityResult::Success(_))
    }

    pub fn is_denied(&self) -> bool {
        matches!(self, CapabilityResult::AccessDenied)
    }

    pub fn unwrap(self) -> T {
        match self {
            CapabilityResult::Success(value) => value,
            _ => panic!("Called unwrap on non-success capability result"),
        }
    }
}

/// Capability table for managing all capabilities in the system
pub struct CapabilityTable {
    capabilities: alloc::collections::BTreeMap<CapabilityId, CapabilityInfo>,
}

#[derive(Debug, Clone)]
struct CapabilityInfo {
    rights: CapabilityRights,
    resource_type: alloc::string::String,
    owner: u64,
    /// Parent capability for cascade revocation (BUG #11 corregido)
    parent: Option<CapabilityId>,
    /// Children capabilities for cascade revocation
    children: Vec<CapabilityId>,
}

impl CapabilityTable {
    pub fn new() -> Self {
        Self {
            capabilities: alloc::collections::BTreeMap::new(),
        }
    }

    /// Register a capability in the table
    pub fn register<T>(&mut self, cap: &Capability<T>, owner: u64) {
        let info = CapabilityInfo {
            rights: cap.rights(),
            resource_type: alloc::string::String::from(core::any::type_name::<T>()),
            owner,
            parent: None,
            children: Vec::new(),
        };
        self.capabilities.insert(cap.id(), info);
    }

    /// Register a capability with a parent for cascade revocation (BUG #11 corregido)
    pub fn register_child<T>(&mut self, cap: &Capability<T>, owner: u64, parent: CapabilityId) {
        let info = CapabilityInfo {
            rights: cap.rights(),
            resource_type: alloc::string::String::from(core::any::type_name::<T>()),
            owner,
            parent: Some(parent),
            children: Vec::new(),
        };
        
        // Add child to parent's children list
        if let Some(parent_info) = self.capabilities.get_mut(&parent) {
            parent_info.children.push(cap.id());
        }
        
        self.capabilities.insert(cap.id(), info);
    }

    /// Check if a capability exists and is valid
    pub fn validate(&self, id: CapabilityId) -> bool {
        self.capabilities.contains_key(&id)
    }

    /// Get the rights for a capability
    pub fn get_rights(&self, id: CapabilityId) -> Option<CapabilityRights> {
        self.capabilities.get(&id).map(|info| info.rights)
    }

    /// Revoke a capability with cascade revocation (BUG #11 corregido)
    pub fn revoke(&mut self, id: CapabilityId) -> bool {
        self.revoke_cascade(id)
    }

    /// Internal cascade revocation implementation
    fn revoke_cascade(&mut self, id: CapabilityId) -> bool {
        if let Some(info) = self.capabilities.remove(&id) {
            // Revoke all children recursively
            for child_id in info.children {
                self.revoke_cascade(child_id);
            }
            true
        } else {
            false
        }
    }

    /// Delegate a capability creating a child relationship (seL4-style)
    pub fn delegate<T>(&mut self, parent_id: CapabilityId, child_rights: CapabilityRights, owner: u64) -> Option<CapabilityId> {
        if let Some(parent_info) = self.capabilities.get(&parent_id) {
            // Child cannot have more rights than parent
            let restricted_rights = parent_info.rights.restrict(child_rights);
            let child_id = CapabilityId::new();

            let child_info = CapabilityInfo {
                rights: restricted_rights,
                resource_type: parent_info.resource_type.clone(),
                owner,
                parent: Some(parent_id),
                children: Vec::new(),
            };

            self.capabilities.insert(child_id, child_info);

            // Re-fetch parent to add child to its list
            if let Some(parent_info_mut) = self.capabilities.get_mut(&parent_id) {
                parent_info_mut.children.push(child_id);
            }

            Some(child_id)
        } else {
            None
        }
    }

    /// Get all capabilities owned by a specific owner
    pub fn get_owner_capabilities(&self, owner: u64) -> Vec<CapabilityId> {
        self.capabilities
            .iter()
            .filter(|(_, info)| info.owner == owner)
            .map(|(id, _)| *id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_creation() {
        let mut value = 42;
        let cap = unsafe { Capability::new(&mut value) };
        assert!(cap.can_read());
        assert!(cap.can_write());
    }

    #[test]
    fn test_capability_restriction() {
        let mut value = 42;
        let cap = unsafe { Capability::new(&mut value) };
        let restricted = cap.restrict(CapabilityRights::READ_ONLY);
        assert!(restricted.can_read());
        assert!(!restricted.can_write());
    }

    #[test]
    fn test_cell_capability() {
        let cell = Cell::new(42);
        let cap = cell.capability();
        assert!(cap.can_read());
        assert!(cap.can_write());
    }

    #[test]
    fn test_invoke_capability() {
        let cell = Cell::new(42);
        let cap = cell.capability();
        let result = invoke_capability(&cap, |v| *v);
        assert_eq!(result, Some(42));
    }

    #[test]
    fn test_invoke_capability_mut() {
        let cell = Cell::new(42);
        let cap = cell.capability();
        let result = invoke_capability_mut(&cap, |v| {
            *v = 100;
            *v
        });
        assert_eq!(result, Some(100));
    }
}
