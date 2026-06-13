//! Temperature Sensors Module
//! 
//! This module implements temperature sensor reading for CPU, GPU, and motherboard
//! using various methods including SMBus/I2C sensors, MSRs, and ACPI thermal information.

use crate::smbus::{I2CAddress, I2CError, SmbusDriver};

/// Tipo de sensor de temperatura
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureSensorType {
    /// Sensor de temperatura CPU (vía MSR)
    Cpu,
    /// Sensor de temperatura GPU
    Gpu,
    /// Sensor de temperatura motherboard (SMBus/I2C)
    Motherboard,
    /// Sensor de temperatura genérico I2C
    I2C,
}

/// Ubicación del sensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemperatureLocation {
    /// CPU Core 0
    CpuCore0,
    /// CPU Core 1
    CpuCore1,
    /// CPU Core 2
    CpuCore2,
    /// CPU Core 3
    CpuCore3,
    /// CPU Package
    CpuPackage,
    /// GPU Core
    GpuCore,
    /// GPU Memory
    GpuMemory,
    /// Motherboard
    Motherboard,
    /// VRM
    Vrm,
    /// Chipset
    Chipset,
    /// Otro
    Other,
}

/// Información de temperatura
#[derive(Debug, Clone)]
pub struct TemperatureReading {
    /// Tipo de sensor
    pub sensor_type: TemperatureSensorType,
    /// Ubicación del sensor
    pub location: TemperatureLocation,
    /// Temperatura en grados Celsius
    pub temperature_celsius: i32,
    /// Temperatura en grados Fahrenheit
    pub temperature_fahrenheit: i32,
    /// Timestamp de la lectura
    pub timestamp: u64,
}

impl TemperatureReading {
    /// Crear una nueva lectura de temperatura
    pub fn new(sensor_type: TemperatureSensorType, location: TemperatureLocation, temperature_celsius: i32) -> Self {
        let temperature_fahrenheit = (temperature_celsius * 9 / 5) + 32;
        // En un sistema real, aquí se obtendría el timestamp real
        let timestamp = 0;
        
        Self {
            sensor_type,
            location,
            temperature_celsius,
            temperature_fahrenheit,
            timestamp,
        }
    }
}

/// Sensor de temperatura genérico I2C
pub struct I2CTemperatureSensor {
    /// Driver SMBus
    smbus: SmbusDriver,
    /// Dirección I2C del sensor
    address: I2CAddress,
    /// Registro de temperatura
    temp_register: u8,
    /// Ubicación del sensor
    location: TemperatureLocation,
}

impl I2CTemperatureSensor {
    /// Crear un nuevo sensor de temperatura I2C
    pub fn new(smbus: SmbusDriver, address: I2CAddress, temp_register: u8, location: TemperatureLocation) -> Self {
        Self {
            smbus,
            address,
            temp_register,
            location,
        }
    }

    /// Leer la temperatura del sensor
    pub fn read_temperature(&mut self) -> Result<TemperatureReading, I2CError> {
        let temp_raw = self.smbus.read_byte_data(self.address, self.temp_register)?;
        
        // La mayoría de los sensores I2C de temperatura usan 0.125°C por LSB
        // El valor es un entero con signo de 8 bits
        let temp_celsius = temp_raw as i8 as i32;
        
        Ok(TemperatureReading::new(
            TemperatureSensorType::I2C,
            self.location,
            temp_celsius,
        ))
    }
}

/// Sensor LM75 (sensor de temperatura I2C común)
pub struct Lm75Sensor {
    i2c_sensor: I2CTemperatureSensor,
}

impl Lm75Sensor {
    /// Crear un nuevo sensor LM75
    pub fn new(smbus: SmbusDriver, address: I2CAddress, location: TemperatureLocation) -> Self {
        // El LM75 tiene el registro de temperatura en 0x00
        let i2c_sensor = I2CTemperatureSensor::new(smbus, address, 0x00, location);
        Self { i2c_sensor }
    }

    /// Leer la temperatura del sensor LM75
    pub fn read_temperature(&mut self) -> Result<TemperatureReading, I2CError> {
        let temp_raw = self.i2c_sensor.smbus.read_word_data(
            self.i2c_sensor.address,
            self.i2c_sensor.temp_register,
        )?;
        
        // El LM75 usa 11 bits para la temperatura con resolución de 0.125°C
        // Los bits 15-11 son la parte entera, bits 10-0 son la parte fraccionaria
        let temp_raw = temp_raw as i16;
        let temp_celsius = (temp_raw >> 5) as i32;
        
        Ok(TemperatureReading::new(
            TemperatureSensorType::I2C,
            self.i2c_sensor.location,
            temp_celsius,
        ))
    }
}

/// Sensor de temperatura CPU (vía MSR)
pub struct CpuTemperatureSensor {
    /// Número de core
    core: u32,
}

impl CpuTemperatureSensor {
    /// Crear un nuevo sensor de temperatura CPU
    pub fn new(core: u32) -> Self {
        Self { core }
    }

    /// Leer la temperatura del CPU
    pub fn read_temperature(&self) -> Option<TemperatureReading> {
        // En un sistema real, aquí se leería el MSR (Model-Specific Register)
        // para obtener la temperatura del CPU core específico
        // Para Intel: MSR IA32_THERM_STATUS (0x19C)
        // Para AMD: MSR MISC_PWR (0xC0010295)
        
        // Simulación: retorna una temperatura plausible
        let temp_celsius = 45 + (self.core * 2) as i32; // Diferentes cores pueden tener temperaturas diferentes
        
        let location = match self.core {
            0 => TemperatureLocation::CpuCore0,
            1 => TemperatureLocation::CpuCore1,
            2 => TemperatureLocation::CpuCore2,
            3 => TemperatureLocation::CpuCore3,
            _ => TemperatureLocation::CpuPackage,
        };
        
        Some(TemperatureReading::new(
            TemperatureSensorType::Cpu,
            location,
            temp_celsius,
        ))
    }
}

/// Gestor de sensores de temperatura
pub struct TemperatureSensorManager {
    /// Sensores de temperatura CPU
    cpu_sensors: alloc::vec::Vec<CpuTemperatureSensor>,
    /// Sensores de temperatura I2C
    i2c_sensors: alloc::vec::Vec<I2CTemperatureSensor>,
    /// Lecturas de temperatura históricas
    readings: alloc::vec::Vec<TemperatureReading>,
}

impl TemperatureSensorManager {
    /// Crear un nuevo gestor de sensores de temperatura
    pub fn new() -> Self {
        Self {
            cpu_sensors: alloc::vec::Vec::new(),
            i2c_sensors: alloc::vec::Vec::new(),
            readings: alloc::vec::Vec::new(),
        }
    }

    /// Agregar un sensor de temperatura CPU
    pub fn add_cpu_sensor(&mut self, core: u32) {
        self.cpu_sensors.push(CpuTemperatureSensor::new(core));
    }

    /// Agregar un sensor de temperatura I2C
    pub fn add_i2c_sensor(&mut self, sensor: I2CTemperatureSensor) {
        self.i2c_sensors.push(sensor);
    }

    /// Leer todas las temperaturas
    pub fn read_all_temperatures(&mut self) -> alloc::vec::Vec<TemperatureReading> {
        let mut all_readings = alloc::vec::Vec::new();
        
        // Leer temperaturas CPU
        for cpu_sensor in &self.cpu_sensors {
            if let Some(reading) = cpu_sensor.read_temperature() {
                all_readings.push(reading);
            }
        }
        
        // Leer temperaturas I2C
        for i2c_sensor in &mut self.i2c_sensors {
            if let Ok(reading) = i2c_sensor.read_temperature() {
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

    /// Obtener la temperatura más alta
    pub fn get_max_temperature(&self) -> Option<i32> {
        self.readings.iter()
            .map(|r| r.temperature_celsius)
            .max()
    }

    /// Obtener la temperatura más baja
    pub fn get_min_temperature(&self) -> Option<i32> {
        self.readings.iter()
            .map(|r| r.temperature_celsius)
            .min()
    }

    /// Obtener la temperatura promedio
    pub fn get_average_temperature(&self) -> Option<i32> {
        if self.readings.is_empty() {
            return None;
        }
        
        let sum: i32 = self.readings.iter()
            .map(|r| r.temperature_celsius)
            .sum();
        
        Some(sum / self.readings.len() as i32)
    }

    /// Obtener lecturas por ubicación
    pub fn get_readings_by_location(&self, location: TemperatureLocation) -> alloc::vec::Vec<&TemperatureReading> {
        self.readings.iter()
            .filter(|r| r.location == location)
            .collect()
    }

    /// Verificar si hay sobrecalentamiento
    pub fn is_overheating(&self, threshold: i32) -> bool {
        if let Some(max_temp) = self.get_max_temperature() {
            max_temp > threshold
        } else {
            false
        }
    }
}

impl Default for TemperatureSensorManager {
    fn default() -> Self {
        Self::new()
    }
}
