//! Voltage Sensors Module
//! 
//! This module implements voltage sensor reading and protection using various methods
//! including SMBus/I2C voltage sensors, ADC readings, and hardware monitoring.

use crate::smbus::{I2CAddress, I2CError, SmbusDriver};

/// Ubicación del sensor de voltaje
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoltageLocation {
    /// CPU Core Voltage
    CpuCore,
    /// CPU VRM
    CpuVrm,
    /// GPU Core Voltage
    GpuCore,
    /// GPU Memory Voltage
    GpuMemory,
    /// RAM Voltage (DDR)
    Ram,
    /// Chipset Voltage
    Chipset,
    /// 3.3V Rail
    Rail3v3,
    /// 5V Rail
    Rail5v,
    /// 12V Rail
    Rail12v,
    /// Otro
    Other,
}

/// Estado del voltaje
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoltageState {
    /// Normal
    Normal,
    /// Bajo (undervoltage)
    Low,
    /// Alto (overvoltage)
    High,
    /// Crítico (peligroso)
    Critical,
    /// Error de lectura
    Error,
}

/// Información de voltaje
#[derive(Debug, Clone)]
pub struct VoltageReading {
    /// Ubicación del sensor
    pub location: VoltageLocation,
    /// Estado del voltaje
    pub state: VoltageState,
    /// Voltaje actual en milivolts
    pub voltage_mv: u32,
    /// Voltaje nominal en milivolts
    pub nominal_mv: u32,
    /// Umbral bajo en milivolts
    pub low_threshold_mv: u32,
    /// Umbral alto en milivolts
    pub high_threshold_mv: u32,
    /// Timestamp de la lectura
    pub timestamp: u64,
}

impl VoltageReading {
    /// Crear una nueva lectura de voltaje
    pub fn new(
        location: VoltageLocation,
        voltage_mv: u32,
        nominal_mv: u32,
        low_threshold_mv: u32,
        high_threshold_mv: u32,
    ) -> Self {
        let state = if voltage_mv < low_threshold_mv {
            VoltageState::Low
        } else if voltage_mv > high_threshold_mv {
            VoltageState::High
        } else {
            VoltageState::Normal
        };
        
        Self {
            location,
            state,
            voltage_mv,
            nominal_mv,
            low_threshold_mv,
            high_threshold_mv,
            timestamp: 0,
        }
    }

    /// Verificar si el voltaje es crítico
    pub fn is_critical(&self) -> bool {
        self.state == VoltageState::Critical
    }

    /// Verificar si el voltaje está fuera de rango
    pub fn is_out_of_range(&self) -> bool {
        self.state == VoltageState::Low || self.state == VoltageState::High
    }
}

/// Sensor de voltaje genérico I2C
pub struct I2CVoltageSensor {
    /// Driver SMBus
    smbus: SmbusDriver,
    /// Dirección I2C del sensor
    address: I2CAddress,
    /// Ubicación del sensor
    location: VoltageLocation,
    /// Registro de voltaje
    voltage_register: u8,
    /// Voltaje nominal en milivolts
    nominal_mv: u32,
    /// Factor de conversión (mV por LSB)
    conversion_factor: f32,
}

impl I2CVoltageSensor {
    /// Crear un nuevo sensor de voltaje I2C
    pub fn new(
        smbus: SmbusDriver,
        address: I2CAddress,
        location: VoltageLocation,
        voltage_register: u8,
        nominal_mv: u32,
        conversion_factor: f32,
    ) -> Self {
        Self {
            smbus,
            address,
            location,
            voltage_register,
            nominal_mv,
            conversion_factor,
        }
    }

    /// Leer el voltaje del sensor
    pub fn read_voltage(&mut self) -> Result<VoltageReading, I2CError> {
        let voltage_raw = self.smbus.read_word_data(self.address, self.voltage_register)?;
        
        // Convertir el valor raw a milivolts
        let voltage_mv = (voltage_raw as f32 * self.conversion_factor) as u32;
        
        // Calcular umbrales (±10% del nominal)
        let low_threshold_mv = (self.nominal_mv as f32 * 0.9) as u32;
        let high_threshold_mv = (self.nominal_mv as f32 * 1.1) as u32;
        
        Ok(VoltageReading::new(
            self.location,
            voltage_mv,
            self.nominal_mv,
            low_threshold_mv,
            high_threshold_mv,
        ))
    }
}

/// Sensor INA219 (sensor de corriente y voltaje I2C común)
pub struct Ina219Sensor {
    i2c_sensor: I2CVoltageSensor,
}

impl Ina219Sensor {
    /// Crear un nuevo sensor INA219
    pub fn new(
        smbus: SmbusDriver,
        address: I2CAddress,
        location: VoltageLocation,
        nominal_mv: u32,
    ) -> Self {
        // El INA219 tiene el registro de bus voltage en 0x02
        // El factor de conversión es 0.125 mV por LSB (configuración por defecto)
        let i2c_sensor = I2CVoltageSensor::new(smbus, address, location, 0x02, nominal_mv, 0.125);
        Self { i2c_sensor }
    }

    /// Leer el voltaje del bus
    pub fn read_bus_voltage(&mut self) -> Result<VoltageReading, I2CError> {
        self.i2c_sensor.read_voltage()
    }

    /// Leer la corriente (en mA)
    pub fn read_current(&mut self) -> Result<i32, I2CError> {
        // El INA219 tiene el registro de current en 0x04
        let current_raw = self.i2c_sensor.smbus.read_word_data(
            self.i2c_sensor.address,
            0x04,
        )?;
        
        // El factor de conversión depende de la configuración del calibración
        // Por defecto, asumimos 0.1 mA por LSB
        let current_ma = (current_raw as i16 as f32 * 0.1) as i32;
        
        Ok(current_ma)
    }

    /// Leer la potencia (en mW)
    pub fn read_power(&mut self) -> Result<u32, I2CError> {
        // El INA219 tiene el registro de power en 0x03
        let power_raw = self.i2c_sensor.smbus.read_word_data(
            self.i2c_sensor.address,
            0x03,
        )?;
        
        // El factor de conversión depende de la configuración del calibración
        // Por defecto, asumimos 2 mW por LSB
        let power_mw = (power_raw as u32) * 2;
        
        Ok(power_mw)
    }
}

/// Gestor de sensores de voltaje
pub struct VoltageSensorManager {
    /// Sensores de voltaje
    voltage_sensors: alloc::vec::Vec<I2CVoltageSensor>,
    /// Lecturas de voltaje históricas
    readings: alloc::vec::Vec<VoltageReading>,
    /// Umbral crítico bajo (porcentaje del nominal)
    critical_low_threshold: f32,
    /// Umbral crítico alto (porcentaje del nominal)
    critical_high_threshold: f32,
}

impl VoltageSensorManager {
    /// Crear un nuevo gestor de sensores de voltaje
    pub fn new() -> Self {
        Self {
            voltage_sensors: alloc::vec::Vec::new(),
            readings: alloc::vec::Vec::new(),
            critical_low_threshold: 0.8, // 80% del nominal
            critical_high_threshold: 1.2, // 120% del nominal
        }
    }

    /// Agregar un sensor de voltaje
    pub fn add_voltage_sensor(&mut self, sensor: I2CVoltageSensor) {
        self.voltage_sensors.push(sensor);
    }

    /// Establecer umbrales críticos
    pub fn set_critical_thresholds(&mut self, low: f32, high: f32) {
        self.critical_low_threshold = low;
        self.critical_high_threshold = high;
    }

    /// Leer todos los voltajes
    pub fn read_all_voltages(&mut self) -> alloc::vec::Vec<VoltageReading> {
        let mut all_readings = alloc::vec::Vec::new();
        
        for sensor in &mut self.voltage_sensors {
            if let Ok(mut reading) = sensor.read_voltage() {
                // Verificar si el voltaje es crítico
                let critical_low = (sensor.nominal_mv as f32 * self.critical_low_threshold) as u32;
                let critical_high = (sensor.nominal_mv as f32 * self.critical_high_threshold) as u32;
                
                if reading.voltage_mv < critical_low || reading.voltage_mv > critical_high {
                    reading.state = VoltageState::Critical;
                }
                
                all_readings.push(reading);
            }
        }
        
        // Guardar lecturas históricas
        self.readings.extend(all_readings.clone());
        
        // Mantener solo las últimas 1000 lecturas
        if self.readings.len() > 1000 {
            self.readings.drain(0..self.readings.len() - 1000);
        }
        
        all_readings
    }

    /// Obtener el voltaje más alto
    pub fn get_max_voltage(&self) -> Option<u32> {
        self.readings.iter()
            .map(|r| r.voltage_mv)
            .max()
    }

    /// Obtener el voltaje más bajo
    pub fn get_min_voltage(&self) -> Option<u32> {
        self.readings.iter()
            .map(|r| r.voltage_mv)
            .min()
    }

    /// Verificar si hay algún voltaje crítico
    pub fn has_critical_voltage(&self) -> bool {
        self.readings.iter()
            .any(|r| r.is_critical())
    }

    /// Obtener lecturas por ubicación
    pub fn get_readings_by_location(&self, location: VoltageLocation) -> alloc::vec::Vec<&VoltageReading> {
        self.readings.iter()
            .filter(|r| r.location == location)
            .collect()
    }

    /// Verificar si hay sobrevoltaje
    pub fn has_overvoltage(&self) -> bool {
        self.readings.iter()
            .any(|r| r.state == VoltageState::High || r.state == VoltageState::Critical)
    }

    /// Verificar si hay subvoltaje
    pub fn has_undervoltage(&self) -> bool {
        self.readings.iter()
            .any(|r| r.state == VoltageState::Low || r.state == VoltageState::Critical)
    }

    /// Obtener voltajes fuera de rango
    pub fn get_out_of_range_voltages(&self) -> alloc::vec::Vec<&VoltageReading> {
        self.readings.iter()
            .filter(|r| r.is_out_of_range())
            .collect()
    }

    /// Calcular el voltaje promedio de una ubicación
    pub fn get_average_voltage(&self, location: VoltageLocation) -> Option<u32> {
        let location_readings: alloc::vec::Vec<_> = self.readings.iter()
            .filter(|r| r.location == location)
            .collect();
        
        if location_readings.is_empty() {
            return None;
        }
        
        let sum: u32 = location_readings.iter()
            .map(|r| r.voltage_mv)
            .sum();
        
        Some(sum / location_readings.len() as u32)
    }
}

impl Default for VoltageSensorManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Sistema de protección de voltaje
pub struct VoltageProtectionSystem {
    /// Gestor de sensores
    sensor_manager: VoltageSensorManager,
    /// Acción a tomar en caso de voltaje crítico
    critical_action: CriticalVoltageAction,
}

/// Acción a tomar en caso de voltaje crítico
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CriticalVoltageAction {
    /// Solo advertir
    WarningOnly,
    /// Throttling del CPU
    ThrottleCpu,
    /// Apagado del sistema
    Shutdown,
}

impl VoltageProtectionSystem {
    /// Crear un nuevo sistema de protección de voltaje
    pub fn new(sensor_manager: VoltageSensorManager, critical_action: CriticalVoltageAction) -> Self {
        Self {
            sensor_manager,
            critical_action,
        }
    }

    /// Verificar voltajes y tomar acción si es necesario
    pub fn check_and_protect(&mut self) -> VoltageProtectionStatus {
        let readings = self.sensor_manager.read_all_voltages();
        
        if self.sensor_manager.has_critical_voltage() {
            match self.critical_action {
                CriticalVoltageAction::WarningOnly => {
                    VoltageProtectionStatus::CriticalWarning
                }
                CriticalVoltageAction::ThrottleCpu => {
                    // En un sistema real, aquí se implementaría el throttling
                    VoltageProtectionStatus::Throttling
                }
                CriticalVoltageAction::Shutdown => {
                    // En un sistema real, aquí se implementaría el apagado
                    VoltageProtectionStatus::Shutdown
                }
            }
        } else if self.sensor_manager.has_overvoltage() {
            VoltageProtectionStatus::Overvoltage
        } else if self.sensor_manager.has_undervoltage() {
            VoltageProtectionStatus::Undervoltage
        } else {
            VoltageProtectionStatus::Normal
        }
    }

    /// Obtener el gestor de sensores
    pub fn sensor_manager(&self) -> &VoltageSensorManager {
        &self.sensor_manager
    }

    /// Obtener el gestor de sensores mutable
    pub fn sensor_manager_mut(&mut self) -> &mut VoltageSensorManager {
        &mut self.sensor_manager
    }
}

/// Estado de protección de voltaje
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VoltageProtectionStatus {
    /// Normal
    Normal,
    /// Sobrevoltaje
    Overvoltage,
    /// Subvoltaje
    Undervoltage,
    /// Advertencia crítica
    CriticalWarning,
    /// Throttling activado
    Throttling,
    /// Apagado iniciado
    Shutdown,
}
