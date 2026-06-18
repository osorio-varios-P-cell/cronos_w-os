//! Hardware Monitor Manager - FASE 4
//!
//! Este módulo unifica todos los sensores de hardware del sistema
//! (temperature_sensors, voltage_sensors, fan_control, smart_data)
//! bajo un único gestor centralizado que se integra con HardwareAwarenessSystem.

use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeType, HardwareType, NodeId};
use crate::hardware_awareness::{HardwareAwarenessSystem, HardwareState, HardwareChangeEvent, ChangeSeverity};

/// Tipo de sensor de hardware
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorType {
    /// Sensor de temperatura
    Temperature,
    /// Sensor de voltaje
    Voltage,
    /// Control de ventilador
    FanControl,
    /// Datos SMART (disco)
    SmartData,
}

/// Estado del sensor
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorState {
    /// No inicializado
    Uninitialized,
    /// Inicializando
    Initializing,
    /// Activo
    Active,
    /// En error
    Error,
    /// Deshabilitado
    Disabled,
}

/// Lectura de sensor
#[derive(Debug, Clone)]
pub struct SensorReading {
    /// ID del sensor
    pub sensor_id: u64,
    /// Tipo de sensor
    pub sensor_type: SensorType,
    /// Valor de la lectura
    pub value: f32,
    /// Unidad de medida
    pub unit: String,
    /// Timestamp
    pub timestamp: u64,
    /// Es crítico
    pub is_critical: bool,
}

/// Información de un sensor
#[derive(Debug, Clone)]
pub struct SensorInfo {
    /// ID único del sensor
    pub id: u64,
    /// Tipo de sensor
    pub sensor_type: SensorType,
    /// Nombre del sensor
    pub name: String,
    /// Estado actual
    pub state: SensorState,
    /// ID del nodo en el grafo
    pub graph_node_id: Option<NodeId>,
    /// Ubicación física
    pub location: String,
}

/// Hardware Monitor Manager - FASE 4
pub struct HardwareMonitorManager {
    /// Graph kernel reference
    graph_kernel: Option<Cell<GraphKernel>>,
    
    /// Sensores de temperatura
    temperature_sensors: BTreeMap<u64, SensorInfo>,
    
    /// Sensores de voltaje
    voltage_sensors: BTreeMap<u64, SensorInfo>,
    
    /// Controles de ventilador
    fan_controls: BTreeMap<u64, SensorInfo>,
    
    /// Datos SMART
    smart_data_sensors: BTreeMap<u64, SensorInfo>,
    
    /// Lecturas recientes
    recent_readings: Vec<SensorReading>,
    
    /// Hardware awareness system
    hardware_awareness: Option<HardwareAwarenessSystem>,
    
    /// Next sensor ID
    next_sensor_id: u64,
    
    /// Estado global del manager
    global_state: SensorState,
    
    /// Intervalo de monitoreo (ms)
    monitoring_interval_ms: u64,
}

impl HardwareMonitorManager {
    /// Crear nuevo Hardware Monitor Manager
    pub fn new() -> Self {
        Self {
            graph_kernel: None,
            temperature_sensors: BTreeMap::new(),
            voltage_sensors: BTreeMap::new(),
            fan_controls: BTreeMap::new(),
            smart_data_sensors: BTreeMap::new(),
            recent_readings: Vec::new(),
            hardware_awareness: None,
            next_sensor_id: 1,
            global_state: SensorState::Uninitialized,
            monitoring_interval_ms: 1000,
        }
    }
    
    /// Establecer el GraphKernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
    }
    
    /// Establecer el Hardware Awareness System
    pub fn set_hardware_awareness(&mut self, hardware_awareness: HardwareAwarenessSystem) {
        self.hardware_awareness = Some(hardware_awareness);
    }
    
    /// Inicializar el manager
    pub fn initialize(&mut self) -> Result<(), String> {
        self.global_state = SensorState::Initializing;
        
        // Registrar sensores predeterminados
        self.register_default_sensors()?;
        
        self.global_state = SensorState::Active;
        Ok(())
    }
    
    /// Registrar sensores predeterminados
    fn register_default_sensors(&mut self) -> Result<(), String> {
        // Sensor de temperatura del CPU
        let cpu_temp_id = self.add_temperature_sensor("CPU Temperature".to_string(), "CPU Package".to_string())?;
        
        // Sensor de temperatura del GPU
        let gpu_temp_id = self.add_temperature_sensor("GPU Temperature".to_string(), "GPU Core".to_string())?;
        
        // Sensor de voltaje del CPU
        let cpu_voltage_id = self.add_voltage_sensor("CPU Voltage".to_string(), "CPU VCore".to_string())?;
        
        // Control de ventilador del CPU
        let cpu_fan_id = self.add_fan_control("CPU Fan".to_string(), "CPU Fan Header".to_string())?;
        
        Ok(())
    }
    
    /// Agregar sensor de temperatura
    pub fn add_temperature_sensor(&mut self, name: String, location: String) -> Result<u64, String> {
        let sensor_id = self.next_sensor_id;
        self.next_sensor_id += 1;
        
        let mut sensor_info = SensorInfo {
            id: sensor_id,
            sensor_type: SensorType::Temperature,
            name,
            state: SensorState::Active,
            graph_node_id: None,
            location,
        };
        
        // Registrar en el grafo si está disponible
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Input);
            let node_name = format!("temp_sensor_{}", sensor_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                sensor_info.graph_node_id = Some(id);
            }
        }
        
        self.temperature_sensors.insert(sensor_id, sensor_info);
        Ok(sensor_id)
    }
    
    /// Agregar sensor de voltaje
    pub fn add_voltage_sensor(&mut self, name: String, location: String) -> Result<u64, String> {
        let sensor_id = self.next_sensor_id;
        self.next_sensor_id += 1;
        
        let mut sensor_info = SensorInfo {
            id: sensor_id,
            sensor_type: SensorType::Voltage,
            name,
            state: SensorState::Active,
            graph_node_id: None,
            location,
        };
        
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Input);
            let node_name = format!("voltage_sensor_{}", sensor_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                sensor_info.graph_node_id = Some(id);
            }
        }
        
        self.voltage_sensors.insert(sensor_id, sensor_info);
        Ok(sensor_id)
    }
    
    /// Agregar control de ventilador
    pub fn add_fan_control(&mut self, name: String, location: String) -> Result<u64, String> {
        let sensor_id = self.next_sensor_id;
        self.next_sensor_id += 1;
        
        let mut sensor_info = SensorInfo {
            id: sensor_id,
            sensor_type: SensorType::FanControl,
            name,
            state: SensorState::Active,
            graph_node_id: None,
            location,
        };
        
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Input);
            let node_name = format!("fan_control_{}", sensor_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                sensor_info.graph_node_id = Some(id);
            }
        }
        
        self.fan_controls.insert(sensor_id, sensor_info);
        Ok(sensor_id)
    }
    
    /// Agregar sensor de datos SMART
    pub fn add_smart_data_sensor(&mut self, name: String, location: String) -> Result<u64, String> {
        let sensor_id = self.next_sensor_id;
        self.next_sensor_id += 1;
        
        let mut sensor_info = SensorInfo {
            id: sensor_id,
            sensor_type: SensorType::SmartData,
            name,
            state: SensorState::Active,
            graph_node_id: None,
            location,
        };
        
        if let Some(ref graph_kernel) = self.graph_kernel {
            let node_type = NodeType::HardwareDevice(HardwareType::Storage);
            let node_name = format!("smart_sensor_{}", sensor_id);
            
            let node_id = invoke_capability_mut(&graph_kernel.capability(), |gk| {
                gk.create_node(node_type, node_name)
            });
            
            if let Some(id) = node_id {
                sensor_info.graph_node_id = Some(id);
            }
        }
        
        self.smart_data_sensors.insert(sensor_id, sensor_info);
        Ok(sensor_id)
    }
    
    /// Leer todos los sensores
    pub fn read_all_sensors(&mut self) -> Vec<SensorReading> {
        let mut readings = Vec::new();
        
        // Leer sensores de temperatura (simulado)
        for (sensor_id, sensor_info) in &self.temperature_sensors {
            let value = self.simulate_temperature_reading(*sensor_id);
            let reading = SensorReading {
                sensor_id: *sensor_id,
                sensor_type: SensorType::Temperature,
                value,
                unit: String::from("°C"),
                timestamp: 0, // Would use real timestamp
                is_critical: value > 80.0,
            };
            readings.push(reading);
        }
        
        // Leer sensores de voltaje (simulado)
        for (sensor_id, sensor_info) in &self.voltage_sensors {
            let value = self.simulate_voltage_reading(*sensor_id);
            let reading = SensorReading {
                sensor_id: *sensor_id,
                sensor_type: SensorType::Voltage,
                value,
                unit: String::from("V"),
                timestamp: 0,
                is_critical: value < 1.0 || value > 1.5,
            };
            readings.push(reading);
        }
        
        // Leer controles de ventilador (simulado)
        for (sensor_id, sensor_info) in &self.fan_controls {
            let value = self.simulate_fan_reading(*sensor_id);
            let reading = SensorReading {
                sensor_id: *sensor_id,
                sensor_type: SensorType::FanControl,
                value,
                unit: String::from("RPM"),
                timestamp: 0,
                is_critical: value < 500.0,
            };
            readings.push(reading);
        }
        
        // Guardar lecturas recientes
        self.recent_readings = readings.clone();
        
        // Notificar a hardware awareness si está disponible
        let hw_awareness_clone = self.hardware_awareness.clone();
        if let Some(mut hw_awareness) = hw_awareness_clone {
            self.update_hardware_awareness(&mut hw_awareness, &readings);
        }
        
        readings
    }
    
    /// Simular lectura de temperatura
    fn simulate_temperature_reading(&self, sensor_id: u64) -> f32 {
        // Simulación básica - en hardware real se leería del sensor
        45.0 + (sensor_id % 20) as f32
    }
    
    /// Simular lectura de voltaje
    fn simulate_voltage_reading(&self, sensor_id: u64) -> f32 {
        // Simulación básica - en hardware real se leería del sensor
        1.2 + ((sensor_id % 5) as f32) * 0.05
    }
    
    /// Simular lectura de ventilador
    fn simulate_fan_reading(&self, sensor_id: u64) -> f32 {
        // Simulación básica - en hardware real se leería del sensor
        1000.0 + (sensor_id % 10) as f32 * 100.0
    }
    
    /// Actualizar hardware awareness con lecturas
    fn update_hardware_awareness(&self, hw_awareness: &mut HardwareAwarenessSystem, readings: &[SensorReading]) {
        // Crear nuevo estado de hardware basado en lecturas
        let mut cpu_temp = 45;
        let mut gpu_temp = 50;
        let mut throttling = false;
        
        for reading in readings {
            match reading.sensor_type {
                SensorType::Temperature => {
                    if reading.value > 80.0 {
                        throttling = true;
                    }
                    if reading.value > 50.0 {
                        cpu_temp = reading.value as i32;
                    } else {
                        gpu_temp = reading.value as i32;
                    }
                }
                _ => {}
            }
        }
        
        let new_state = crate::hardware_awareness::HardwareState {
            cpu_temperature: cpu_temp,
            gpu_temperature: gpu_temp,
            motherboard_temperature: 40,
            cpu_voltage: 1200,
            voltage_12v: 12000,
            voltage_5v: 5000,
            voltage_3v3: 3300,
            cpu_fan_speed: 1200,
            system_fan_speed: 1000,
            throttling_active: throttling,
            throttle_factor: if throttling { 0.8 } else { 1.0 },
            drive_health: 95,
            overall_health: if throttling { 70 } else { 95 },
            timestamp: 0,
        };
        
        hw_awareness.update_state(new_state);
    }
    
    /// Obtener lecturas recientes
    pub fn get_recent_readings(&self) -> &[SensorReading] {
        &self.recent_readings
    }
    
    /// Obtener sensores por tipo
    pub fn get_sensors_by_type(&self, sensor_type: SensorType) -> Vec<&SensorInfo> {
        match sensor_type {
            SensorType::Temperature => self.temperature_sensors.values().collect(),
            SensorType::Voltage => self.voltage_sensors.values().collect(),
            SensorType::FanControl => self.fan_controls.values().collect(),
            SensorType::SmartData => self.smart_data_sensors.values().collect(),
        }
    }
    
    /// Obtener número total de sensores
    pub fn total_sensor_count(&self) -> usize {
        self.temperature_sensors.len() + 
        self.voltage_sensors.len() + 
        self.fan_controls.len() + 
        self.smart_data_sensors.len()
    }
    
    /// Detener todos los sensores
    pub fn shutdown(&mut self) -> Result<(), String> {
        self.global_state = SensorState::Disabled;
        
        // Limpiar todos los sensores
        self.temperature_sensors.clear();
        self.voltage_sensors.clear();
        self.fan_controls.clear();
        self.smart_data_sensors.clear();
        self.recent_readings.clear();
        
        Ok(())
    }
}

impl Default for HardwareMonitorManager {
    fn default() -> Self {
        Self::new()
    }
}
