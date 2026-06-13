//! SMART Data Module
//! 
//! This module implements SMART (Self-Monitoring, Analysis and Reporting Technology)
//! data reading for storage health monitoring, supporting both SATA/AHCI and NVMe drives.

extern crate alloc;

use alloc::string::String;

/// Tipo de drive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveType {
    /// SATA/AHCI
    Sata,
    /// NVMe
    NvMe,
}

/// Estado de salud del drive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DriveHealth {
    /// Buen estado
    Good,
    /// Advertencia
    Warning,
    /// Crítico
    Critical,
    /// Fallando
    Failing,
    /// Desconocido
    Unknown,
}

/// Atributo SMART
#[derive(Debug, Clone)]
pub struct SmartAttribute {
    /// ID del atributo
    pub id: u8,
    /// Nombre del atributo
    pub name: String,
    /// Valor actual
    pub value: u8,
    /// Valor peor
    pub worst: u8,
    /// Umbral
    pub threshold: u8,
    /// Datos raw
    pub raw: u64,
    /// Es prefallo
    pub is_prefail: bool,
}

impl SmartAttribute {
    /// Verificar si el atributo indica un problema
    pub fn is_failing(&self) -> bool {
        self.is_prefail && self.value <= self.threshold
    }
}

/// Información SMART del drive
#[derive(Debug, Clone)]
pub struct SmartInfo {
    /// Tipo de drive
    pub drive_type: DriveType,
    /// Modelo del drive
    pub model: String,
    /// Número de serie
    pub serial: String,
    /// Firmware
    pub firmware: String,
    /// Capacidad en bytes
    pub capacity: u64,
    /// Horas de funcionamiento
    pub power_on_hours: u64,
    /// Cantidad de ciclos de encendido
    pub power_cycle_count: u64,
    /// Temperatura actual en Celsius
    pub temperature: i32,
    /// Atributos SMART
    pub attributes: alloc::vec::Vec<SmartAttribute>,
    /// Estado de salud
    pub health: DriveHealth,
    /// Porcentaje de salud (0-100)
    pub health_percentage: u8,
    /// Timestamp de la lectura
    pub timestamp: u64,
}

impl SmartInfo {
    /// Crear nueva información SMART
    pub fn new(drive_type: DriveType) -> Self {
        Self {
            drive_type,
            model: String::new(),
            serial: String::new(),
            firmware: String::new(),
            capacity: 0,
            power_on_hours: 0,
            power_cycle_count: 0,
            temperature: 0,
            attributes: alloc::vec::Vec::new(),
            health: DriveHealth::Unknown,
            health_percentage: 100,
            timestamp: 0,
        }
    }

    /// Calcular el estado de salud basado en atributos
    pub fn calculate_health(&mut self) {
        let mut failing_count = 0;
        let mut warning_count = 0;
        
        for attr in &self.attributes {
            if attr.is_failing() {
                failing_count += 1;
            } else if attr.is_prefail && attr.value <= attr.threshold + 5 {
                warning_count += 1;
            }
        }
        
        self.health = if failing_count > 0 {
            DriveHealth::Failing
        } else if warning_count > 2 {
            DriveHealth::Critical
        } else if warning_count > 0 {
            DriveHealth::Warning
        } else {
            DriveHealth::Good
        };
        
        // Calcular porcentaje de salud
        self.health_percentage = match self.health {
            DriveHealth::Good => 100,
            DriveHealth::Warning => 75,
            DriveHealth::Critical => 50,
            DriveHealth::Failing => 25,
            DriveHealth::Unknown => 0,
        };
    }

    /// Verificar si el drive está fallando
    pub fn is_failing(&self) -> bool {
        self.health == DriveHealth::Failing
    }

    /// Verificar si el drive necesita atención
    pub fn needs_attention(&self) -> bool {
        self.health != DriveHealth::Good
    }
}

/// Comando SMART para SATA
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum SataSmartCommand {
    /// Leer SMART data
    ReadSmartData = 0xB0,
    /// Leer SMART thresholds
    ReadSmartThresholds = 0xD1,
}

/// Lector SMART para SATA/AHCI
pub struct SataSmartReader {
    /// Puerto del drive
    pub port: u8,
}

impl SataSmartReader {
    /// Crear un nuevo lector SMART SATA
    pub fn new(port: u8) -> Self {
        Self { port }
    }

    /// Leer información SMART del drive
    pub fn read_smart_info(&self) -> Result<SmartInfo, &'static str> {
        // En un sistema real, aquí se:
        // 1. Enviar comando IDENTIFY DEVICE para obtener información del drive
        // 2. Enviar comando SMART READ DATA para obtener atributos SMART
        // 3. Enviar comando SMART READ THRESHOLDS para obtener umbrales
        // 4. Parsear los datos y crear la estructura SmartInfo
        
        let mut smart_info = SmartInfo::new(DriveType::Sata);
        
        // Simulación: datos de ejemplo
        smart_info.model = String::from("ST1000DM003-1CH162");
        smart_info.serial = String::from("W1F12345");
        smart_info.firmware = String::from("CC45");
        smart_info.capacity = 1_000_204_886_016; // 1 TB
        smart_info.power_on_hours = 8760; // 1 año
        smart_info.power_cycle_count = 500;
        smart_info.temperature = 35;
        
        // Atributos SMART comunes (simulados)
        smart_info.attributes.push(SmartAttribute {
            id: 1,
            name: String::from("Raw Read Error Rate"),
            value: 100,
            worst: 100,
            threshold: 6,
            raw: 0,
            is_prefail: true,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 3,
            name: String::from("Spin Up Time"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 0,
            is_prefail: false,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 5,
            name: String::from("Reallocated Sectors Count"),
            value: 100,
            worst: 100,
            threshold: 10,
            raw: 0,
            is_prefail: true,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 9,
            name: String::from("Power On Hours"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 8760,
            is_prefail: false,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 12,
            name: String::from("Power Cycle Count"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 500,
            is_prefail: false,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 194,
            name: String::from("Temperature"),
            value: 100,
            worst: 95,
            threshold: 0,
            raw: 35,
            is_prefail: false,
        });
        
        smart_info.calculate_health();
        
        Ok(smart_info)
    }
}

/// Lector SMART para NVMe
pub struct NvmeSmartReader {
    /// Namespace del drive
    pub namespace_id: u32,
}

impl NvmeSmartReader {
    /// Crear un nuevo lector SMART NVMe
    pub fn new(namespace_id: u32) -> Self {
        Self { namespace_id }
    }

    /// Leer información SMART del drive NVMe
    pub fn read_smart_info(&self) -> Result<SmartInfo, &'static str> {
        // En un sistema real, aquí se:
        // 1. Enviar comando NVMe Identify para obtener información del drive
        // 2. Enviar comando NVMe Get Log Page con SMART/Health Information Log
        // 3. Parsear los datos y crear la estructura SmartInfo
        
        let mut smart_info = SmartInfo::new(DriveType::NvMe);
        
        // Simulación: datos de ejemplo
        smart_info.model = String::from("Samsung 970 EVO 1TB");
        smart_info.serial = String::from("S4EWNX0M123456");
        smart_info.firmware = String::from("2B2QEXE7");
        smart_info.capacity = 1_000_204_886_016; // 1 TB
        smart_info.power_on_hours = 4380; // 6 meses
        smart_info.power_cycle_count = 200;
        smart_info.temperature = 40;
        
        // Atributos SMART NVMe (simulados)
        smart_info.attributes.push(SmartAttribute {
            id: 1,
            name: String::from("Available Spare"),
            value: 100,
            worst: 100,
            threshold: 10,
            raw: 100,
            is_prefail: true,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 2,
            name: String::from("Available Spare Threshold"),
            value: 100,
            worst: 100,
            threshold: 10,
            raw: 10,
            is_prefail: true,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 3,
            name: String::from("Percentage Used"),
            value: 95,
            worst: 95,
            threshold: 100,
            raw: 5,
            is_prefail: true,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 4,
            name: String::from("Data Units Read"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 1000000,
            is_prefail: false,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 5,
            name: String::from("Data Units Written"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 500000,
            is_prefail: false,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 6,
            name: String::from("Media Errors"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 0,
            is_prefail: true,
        });
        
        smart_info.attributes.push(SmartAttribute {
            id: 7,
            name: String::from("Number of Error Log Entries"),
            value: 100,
            worst: 100,
            threshold: 0,
            raw: 0,
            is_prefail: false,
        });
        
        smart_info.calculate_health();
        
        Ok(smart_info)
    }
}

/// Gestor de datos SMART
pub struct SmartDataManager {
    /// Lectores SATA
    sata_readers: alloc::vec::Vec<SataSmartReader>,
    /// Lectores NVMe
    nvme_readers: alloc::vec::Vec<NvmeSmartReader>,
    /// Información SMART histórica
    smart_history: alloc::vec::Vec<SmartInfo>,
}

impl SmartDataManager {
    /// Crear un nuevo gestor de datos SMART
    pub fn new() -> Self {
        Self {
            sata_readers: alloc::vec::Vec::new(),
            nvme_readers: alloc::vec::Vec::new(),
            smart_history: alloc::vec::Vec::new(),
        }
    }

    /// Agregar un lector SATA
    pub fn add_sata_reader(&mut self, reader: SataSmartReader) {
        self.sata_readers.push(reader);
    }

    /// Agregar un lector NVMe
    pub fn add_nvme_reader(&mut self, reader: NvmeSmartReader) {
        self.nvme_readers.push(reader);
    }

    /// Leer información SMART de todos los drives
    pub fn read_all_smart_info(&mut self) -> alloc::vec::Vec<SmartInfo> {
        let mut all_info = alloc::vec::Vec::new();
        
        // Leer información SATA
        for reader in &self.sata_readers {
            if let Ok(info) = reader.read_smart_info() {
                all_info.push(info);
            }
        }
        
        // Leer información NVMe
        for reader in &self.nvme_readers {
            if let Ok(info) = reader.read_smart_info() {
                all_info.push(info);
            }
        }
        
        // Guardar historial
        self.smart_history.extend(all_info.clone());
        
        // Mantener solo las últimas 1000 lecturas
        if self.smart_history.len() > 1000 {
            self.smart_history.drain(0..self.smart_history.len() - 1000);
        }
        
        all_info
    }

    /// Verificar si algún drive está fallando
    pub fn has_failing_drive(&self) -> bool {
        self.smart_history.iter()
            .any(|info| info.is_failing())
    }

    /// Obtener drives que necesitan atención
    pub fn get_drives_needing_attention(&self) -> alloc::vec::Vec<&SmartInfo> {
        self.smart_history.iter()
            .filter(|info| info.needs_attention())
            .collect()
    }

    /// Obtener el porcentaje de salud promedio
    pub fn get_average_health_percentage(&self) -> Option<u8> {
        if self.smart_history.is_empty() {
            return None;
        }
        
        let sum: u8 = self.smart_history.iter()
            .map(|info| info.health_percentage)
            .fold(0, |acc, x| acc.saturating_add(x));
        
        Some(sum / self.smart_history.len() as u8)
    }

    /// Obtener información por tipo de drive
    pub fn get_info_by_type(&self, drive_type: DriveType) -> alloc::vec::Vec<&SmartInfo> {
        self.smart_history.iter()
            .filter(|info| info.drive_type == drive_type)
            .collect()
    }

    /// Obtener atributos específicos de todos los drives
    pub fn get_attribute_by_id(&self, attribute_id: u8) -> alloc::vec::Vec<&SmartAttribute> {
        let mut attributes = alloc::vec::Vec::new();
        
        for info in &self.smart_history {
            for attr in &info.attributes {
                if attr.id == attribute_id {
                    attributes.push(attr);
                }
            }
        }
        
        attributes
    }
}

impl Default for SmartDataManager {
    fn default() -> Self {
        Self::new()
    }
}
