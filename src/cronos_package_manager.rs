//! Package Manager de CRONOS original adaptado a CRONOS W-OS
//!
//! Este módulo incorpora el sistema de gestión de paquetes de CRONOS original,
//! adaptado al sistema de capabilities y arquitectura de exokernel con grafos

use core::fmt;
use alloc::vec::Vec;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::collections::BTreeMap;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::GraphKernel;

/// Estado de un paquete
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageState {
    NotInstalled,
    Installed,
    UpdateAvailable,
    Broken,
}

/// Dependencia de un paquete
#[derive(Debug, Clone)]
pub struct PackageDependency {
    pub name: String,
    pub version: String,
}

/// Información de un paquete
#[derive(Debug, Clone)]
pub struct Package {
    pub package_id: u64,
    pub name: String,
    pub version: String,
    pub description: String,
    pub size: u64,
    pub dependencies: Vec<PackageDependency>,
    pub state: PackageState,
    pub install_path: String,
    pub capability_id: Option<CapabilityId>,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl Package {
    pub fn new(package_id: u64, name: &str, version: &str, description: &str, size: u64, install_path: &str) -> Self {
        Self {
            package_id,
            name: String::from(name),
            version: String::from(version),
            description: String::from(description),
            size,
            dependencies: Vec::new(),
            state: PackageState::NotInstalled,
            install_path: String::from(install_path),
            capability_id: None,
            graph_node_id: None,
        }
    }

    pub fn add_dependency(&mut self, name: &str, version: &str) {
        self.dependencies.push(PackageDependency {
            name: String::from(name),
            version: String::from(version),
        });
    }

    pub fn set_state(&mut self, state: PackageState) {
        self.state = state;
    }

    pub fn is_installed(&self) -> bool {
        self.state == PackageState::Installed
    }

    pub fn is_not_installed(&self) -> bool {
        self.state == PackageState::NotInstalled
    }

    pub fn has_update_available(&self) -> bool {
        self.state == PackageState::UpdateAvailable
    }

    pub fn is_broken(&self) -> bool {
        self.state == PackageState::Broken
    }
}

/// Repositorio de paquetes
#[derive(Debug, Clone)]
pub struct PackageRepository {
    pub repository_id: u64,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub graph_node_id: Option<crate::graph_kernel::NodeId>,
}

impl PackageRepository {
    pub fn new(repository_id: u64, name: &str, url: &str) -> Self {
        Self {
            repository_id,
            name: String::from(name),
            url: String::from(url),
            enabled: true,
            graph_node_id: None,
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Gestor de paquetes
#[derive(Debug, Clone)]
pub struct CronosPackageManager {
    pub packages: BTreeMap<u64, Package>,
    pub repositories: BTreeMap<u64, PackageRepository>,
    pub next_package_id: u64,
    pub next_repository_id: u64,
    pub graph_kernel: Option<Cell<GraphKernel>>,
}

impl CronosPackageManager {
    pub fn new() -> Self {
        Self {
            packages: BTreeMap::new(),
            repositories: BTreeMap::new(),
            next_package_id: 1,
            next_repository_id: 1,
            graph_kernel: None,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }

    /// Agregar repositorio
    pub fn add_repository(&mut self, name: &str, url: &str) -> u64 {
        let repository_id = self.next_repository_id;
        self.next_repository_id += 1;

        let mut repo = PackageRepository::new(repository_id, name, url);

        // Registrar el repositorio como nodo en el grafo
        if let Some(ref graph_kernel) = self.graph_kernel {
            use crate::graph_kernel::{NodeType, GraphKernel};
            let node_type = NodeType::File;
            let node_name = format!("repository_{}", name);
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            repo.graph_node_id = node_id;
        }

        self.repositories.insert(repository_id, repo);
        repository_id
    }

    /// Remover repositorio
    pub fn remove_repository(&mut self, repository_id: u64) -> Result<(), String> {
        if self.repositories.remove(&repository_id).is_some() {
            Ok(())
        } else {
            Err(format!("Repository {} not found", repository_id))
        }
    }

    /// Obtener repositorio
    pub fn get_repository(&self, repository_id: u64) -> Option<&PackageRepository> {
        self.repositories.get(&repository_id)
    }

    /// Obtener repositorio mut
    pub fn get_repository_mut(&mut self, repository_id: u64) -> Option<&mut PackageRepository> {
        self.repositories.get_mut(&repository_id)
    }

    /// Listar repositorios
    pub fn list_repositories(&self) -> Vec<&PackageRepository> {
        self.repositories.values().collect()
    }

    /// Listar repositorios habilitados
    pub fn list_enabled_repositories(&self) -> Vec<&PackageRepository> {
        self.repositories.values().filter(|r| r.is_enabled()).collect()
    }

    /// Agregar paquete
    pub fn add_package(&mut self, package: Package) {
        self.packages.insert(package.package_id, package);
    }

    /// Remover paquete
    pub fn remove_package(&mut self, package_id: u64) -> Result<(), String> {
        if self.packages.remove(&package_id).is_some() {
            Ok(())
        } else {
            Err(format!("Package {} not found", package_id))
        }
    }

    /// Obtener paquete
    pub fn get_package(&self, package_id: u64) -> Option<&Package> {
        self.packages.get(&package_id)
    }

    /// Obtener paquete por nombre
    pub fn get_package_by_name(&self, name: &str) -> Option<&Package> {
        self.packages.values().find(|p| p.name == name)
    }

    /// Obtener paquete mut
    pub fn get_package_mut(&mut self, package_id: u64) -> Option<&mut Package> {
        self.packages.get_mut(&package_id)
    }

    /// Listar paquetes
    pub fn list_packages(&self) -> Vec<&Package> {
        self.packages.values().collect()
    }

    /// Listar paquetes instalados
    pub fn list_installed_packages(&self) -> Vec<&Package> {
        self.packages.values().filter(|p| p.is_installed()).collect()
    }

    /// Listar paquetes disponibles
    pub fn list_available_packages(&self) -> Vec<&Package> {
        self.packages.values().filter(|p| p.is_not_installed()).collect()
    }

    /// Listar paquetes actualizables
    pub fn list_updatable_packages(&self) -> Vec<&Package> {
        self.packages.values().filter(|p| p.has_update_available()).collect()
    }

    /// Contar paquetes
    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    /// Contar paquetes instalados
    pub fn installed_package_count(&self) -> usize {
        self.packages.values().filter(|p| p.is_installed()).count()
    }

    /// Instalar paquete
    pub fn install_package(&mut self, name: &str) -> Result<(), String> {
        // Primero verificar dependencias antes de hacer el borrow mutable
        let dependencies: Vec<(String, String)> = if let Some(package) = self.get_package_by_name(name) {
            package.dependencies.iter().map(|d| (d.name.clone(), d.version.clone())).collect()
        } else {
            return Err(format!("Package {} not found", name));
        };

        for (dep_name, _) in dependencies {
            if let Some(dep_package) = self.get_package_by_name(&dep_name) {
                if !dep_package.is_installed() {
                    return Err(format!("Dependency not installed: {}", dep_name));
                }
            } else {
                return Err(format!("Dependency not found: {}", dep_name));
            }
        }

        if let Some(package) = self.packages.values_mut().find(|p| p.name == name) {
            if package.is_installed() {
                return Err(String::from("Package is already installed"));
            }

            // En un sistema real, aquí se instalaría el paquete
            package.set_state(PackageState::Installed);
            Ok(())
        } else {
            Err(format!("Package {} not found", name))
        }
    }

    /// Desinstalar paquete
    pub fn uninstall_package(&mut self, name: &str) -> Result<(), String> {
        if let Some(package) = self.packages.values_mut().find(|p| p.name == name) {
            if !package.is_installed() {
                return Err(String::from("Package is not installed"));
            }

            // En un sistema real, aquí se desinstalaría el paquete
            package.set_state(PackageState::NotInstalled);
            Ok(())
        } else {
            Err(format!("Package {} not found", name))
        }
    }

    /// Actualizar paquete
    pub fn update_package(&mut self, name: &str) -> Result<(), String> {
        if let Some(package) = self.packages.values_mut().find(|p| p.name == name) {
            if !package.is_installed() {
                return Err(String::from("Package is not installed"));
            }

            if !package.has_update_available() {
                return Err(String::from("No update available"));
            }

            // En un sistema real, aquí se actualizaría el paquete
            package.set_state(PackageState::Installed);
            Ok(())
        } else {
            Err(format!("Package {} not found", name))
        }
    }

    /// Actualizar todos los paquetes
    pub fn update_all_packages(&mut self) -> Result<usize, String> {
        let updatable_names: Vec<String> = self.list_updatable_packages()
            .iter()
            .map(|p| p.name.clone())
            .collect();
        
        let mut updated_count = 0;

        for name in updatable_names {
            if self.update_package(&name).is_ok() {
                updated_count += 1;
            }
        }

        Ok(updated_count)
    }

    /// Buscar paquetes
    pub fn search_packages(&self, query: &str) -> Vec<&Package> {
        self.packages.values()
            .filter(|p| p.name.to_lowercase().contains(&query.to_lowercase()) ||
                       p.description.to_lowercase().contains(&query.to_lowercase()))
            .collect()
    }

    /// Obtener dependencias de un paquete
    pub fn get_package_dependencies(&self, name: &str) -> Option<Vec<String>> {
        self.get_package_by_name(name).map(|p| {
            p.dependencies.iter().map(|d| d.name.clone()).collect()
        })
    }

    /// Obtener paquetes dependientes
    pub fn get_dependent_packages(&self, name: &str) -> Vec<&Package> {
        self.packages.values()
            .filter(|p| p.dependencies.iter().any(|d| d.name == name))
            .collect()
    }
}

impl Default for CronosPackageManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores del gestor de paquetes
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageManagerError {
    PackageNotFound,
    AlreadyInstalled,
    NotInstalled,
    NoUpdateAvailable,
    DependencyNotFound(String),
    DependencyNotInstalled(String),
    DownloadFailed,
    InstallationFailed,
    UninstallationFailed,
    CorruptedPackage,
}

impl fmt::Display for PackageManagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackageManagerError::PackageNotFound => write!(f, "Package not found"),
            PackageManagerError::AlreadyInstalled => write!(f, "Package is already installed"),
            PackageManagerError::NotInstalled => write!(f, "Package is not installed"),
            PackageManagerError::NoUpdateAvailable => write!(f, "No update available"),
            PackageManagerError::DependencyNotFound(name) => write!(f, "Dependency not found: {}", name),
            PackageManagerError::DependencyNotInstalled(name) => write!(f, "Dependency not installed: {}", name),
            PackageManagerError::DownloadFailed => write!(f, "Failed to download package"),
            PackageManagerError::InstallationFailed => write!(f, "Failed to install package"),
            PackageManagerError::UninstallationFailed => write!(f, "Failed to uninstall package"),
            PackageManagerError::CorruptedPackage => write!(f, "Package is corrupted"),
        }
    }
}
