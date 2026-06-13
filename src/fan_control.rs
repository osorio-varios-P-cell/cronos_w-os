//! Fan Control Module
//! 
//! This module implements fan speed sensors and control using various methods
//! including SMBus/I2C fan controllers, PWM control, and RPM reading.

use crate::smbus::{I2CAddress, I2CError, SmbusDriver};

/// Tipo de control de fan
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanControlType {
    /// PWM (Pulse Width Modulation)
    Pwm,
    /// DC (control de voltaje)
    Dc,
    /// Automático (basado en temperatura)
    Auto,
}

/// Ubicación del fan
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanLocation {
    /// CPU Fan
    Cpu,
    /// GPU Fan
    Gpu,
    /// Case Fan (frontal)
    CaseFront,
    /// Case Fan (trasero)
    CaseRear,
    /// Case Fan (superior)
    CaseTop,
    /// Case Fan (inferior)
    CaseBottom,
    /// Fan del chipset
    Chipset,
    /// Fan del VRM
    Vrm,
    /// Otro
    Other,
}

/// Estado del fan
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FanState {
    /// Detenido
    Stopped,
    /// Funcionando
    Running,
    /// Error
    Error,
}

/// Información del fan
#[derive(Debug, Clone)]
pub struct FanInfo {
    /// Ubicación del fan
    pub location: FanLocation,
    /// Tipo de control
    pub control_type: FanControlType,
    /// Estado actual
    pub state: FanState,
    /// Velocidad actual en RPM
    pub rpm: u32,
    /// Velocidad PWM actual (0-255)
    pub pwm_speed: u8,
    /// Velocidad mínima PWM
    pub min_pwm: u8,
    /// Velocidad máxima PWM
    pub max_pwm: u8,
    /// Timestamp de la lectura
    pub timestamp: u64,
}

impl FanInfo {
    /// Crear nueva información de fan
    pub fn new(location: FanLocation, control_type: FanControlType) -> Self {
        Self {
            location,
            control_type,
            state: FanState::Stopped,
            rpm: 0,
            pwm_speed: 0,
            min_pwm: 0,
            max_pwm: 255,
            timestamp: 0,
        }
    }
}

/// Sensor de fan genérico I2C
pub struct I2CFanController {
    /// Driver SMBus
    smbus: SmbusDriver,
    /// Dirección I2C del controlador
    address: I2CAddress,
    /// Ubicación del fan
    location: FanLocation,
    /// Registro de velocidad del fan
    fan_speed_register: u8,
    /// Registro de control PWM
    pwm_control_register: u8,
}

impl I2CFanController {
    /// Crear un nuevo controlador de fan I2C
    pub fn new(
        smbus: SmbusDriver,
        address: I2CAddress,
        location: FanLocation,
        fan_speed_register: u8,
        pwm_control_register: u8,
    ) -> Self {
        Self {
            smbus,
            address,
            location,
            fan_speed_register,
            pwm_control_register,
        }
    }

    /// Leer la velocidad del fan en RPM
    pub fn read_fan_rpm(&mut self) -> Result<u32, I2CError> {
        // En un sistema real, aquí se leerían 2 bytes del registro de velocidad
        // y se convertiría a RPM usando la fórmula específica del chip
        let speed_raw = self.smbus.read_word_data(self.address, self.fan_speed_register)?;
        
        // Fórmula genérica: RPM = (speed_raw * divisor) / pulsos_por_rev
        // Esto varía según el chip específico
        let rpm = (speed_raw as u32 * 60) / 2; // Simulación
        
        Ok(rpm)
    }

    /// Establecer la velocidad PWM del fan
    pub fn set_pwm_speed(&mut self, pwm: u8) -> Result<(), I2CError> {
        self.smbus.write_byte_data(self.address, self.pwm_control_register, pwm)
    }

    /// Leer la velocidad PWM actual
    pub fn read_pwm_speed(&mut self) -> Result<u8, I2CError> {
        self.smbus.read_byte_data(self.address, self.pwm_control_register)
    }

    /// Obtener información completa del fan
    pub fn get_fan_info(&mut self) -> Result<FanInfo, I2CError> {
        let rpm = self.read_fan_rpm()?;
        let pwm_speed = self.read_pwm_speed()?;
        
        let state = if rpm > 0 {
            FanState::Running
        } else {
            FanState::Stopped
        };
        
        Ok(FanInfo {
            location: self.location,
            control_type: FanControlType::Pwm,
            state,
            rpm,
            pwm_speed,
            min_pwm: 0,
            max_pwm: 255,
            timestamp: 0,
        })
    }
}

/// Controlador de fan EMC2101 (chip común de control de fans)
pub struct Emc2101Controller {
    i2c_controller: I2CFanController,
}

impl Emc2101Controller {
    /// Crear un nuevo controlador EMC2101
    pub fn new(smbus: SmbusDriver, address: I2CAddress, location: FanLocation) -> Self {
        // El EMC2101 tiene:
        // - Registro de velocidad del fan: 0x29 (TACH reading)
        // - Registro de control PWM: 0x30 (PWM output)
        let i2c_controller = I2CFanController::new(smbus, address, location, 0x29, 0x30);
        Self { i2c_controller }
    }

    /// Leer la velocidad del fan en RPM
    pub fn read_fan_rpm(&mut self) -> Result<u32, I2CError> {
        self.i2c_controller.read_fan_rpm()
    }

    /// Establecer la velocidad PWM del fan (0-255)
    pub fn set_pwm_speed(&mut self, pwm: u8) -> Result<(), I2CError> {
        self.i2c_controller.set_pwm_speed(pwm)
    }

    /// Obtener información del fan
    pub fn get_fan_info(&mut self) -> Result<FanInfo, I2CError> {
        self.i2c_controller.get_fan_info()
    }
}

/// Curva de control de fan
#[derive(Debug, Clone)]
pub struct FanCurve {
    /// Puntos de la curva (temperatura, velocidad PWM)
    pub points: alloc::vec::Vec<(i32, u8)>,
}

impl FanCurve {
    /// Crear una nueva curva de fan
    pub fn new() -> Self {
        Self {
            points: alloc::vec::Vec::new(),
        }
    }

    /// Agregar un punto a la curva
    pub fn add_point(&mut self, temperature: i32, pwm: u8) {
        self.points.push((temperature, pwm));
        self.points.sort_by_key(|(t, _)| *t);
    }

    /// Calcular la velocidad PWM basada en la temperatura
    pub fn calculate_pwm(&self, temperature: i32) -> u8 {
        if self.points.is_empty() {
            return 128; // 50% por defecto
        }

        // Si la temperatura está por debajo del primer punto
        if temperature <= self.points[0].0 {
            return self.points[0].1;
        }

        // Si la temperatura está por encima del último punto
        if temperature >= self.points.last().unwrap().0 {
            return self.points.last().unwrap().1;
        }

        // Interpolación lineal entre puntos
        for i in 0..self.points.len() - 1 {
            let (temp1, pwm1) = self.points[i];
            let (temp2, pwm2) = self.points[i + 1];

            if temperature >= temp1 && temperature <= temp2 {
                let temp_range = temp2 - temp1;
                let pwm_range = pwm2 as i32 - pwm1 as i32;
                let temp_offset = temperature - temp1;
                let pwm = pwm1 as i32 + (temp_offset * pwm_range / temp_range);
                return pwm as u8;
            }
        }

        128 // Valor por defecto
    }
}

impl Default for FanCurve {
    fn default() -> Self {
        let mut curve = Self::new();
        // Curva por defecto típica
        curve.add_point(30, 0);   // 30°C -> 0% (fan apagado)
        curve.add_point(40, 64);  // 40°C -> 25%
        curve.add_point(50, 128); // 50°C -> 50%
        curve.add_point(60, 192); // 60°C -> 75%
        curve.add_point(70, 255); // 70°C -> 100%
        curve
    }
}

/// Gestor de control de fans
pub struct FanControllerManager {
    /// Controladores de fans
    fan_controllers: alloc::vec::Vec<I2CFanController>,
    /// Curvas de control por ubicación
    fan_curves: alloc::vec::Vec<(FanLocation, FanCurve)>,
    /// Información histórica de fans
    fan_history: alloc::vec::Vec<FanInfo>,
}

impl FanControllerManager {
    /// Crear un nuevo gestor de control de fans
    pub fn new() -> Self {
        Self {
            fan_controllers: alloc::vec::Vec::new(),
            fan_curves: alloc::vec::Vec::new(),
            fan_history: alloc::vec::Vec::new(),
        }
    }

    /// Agregar un controlador de fan
    pub fn add_fan_controller(&mut self, controller: I2CFanController) {
        self.fan_controllers.push(controller);
    }

    /// Establecer una curva de control para una ubicación
    pub fn set_fan_curve(&mut self, location: FanLocation, curve: FanCurve) {
        // Eliminar curva existente si la hay
        self.fan_curves.retain(|(loc, _)| loc != &location);
        self.fan_curves.push((location, curve));
    }

    /// Obtener la curva de control para una ubicación
    pub fn get_fan_curve(&self, location: FanLocation) -> Option<&FanCurve> {
        self.fan_curves.iter()
            .find(|(loc, _)| loc == &location)
            .map(|(_, curve)| curve)
    }

    /// Leer información de todos los fans
    pub fn read_all_fans(&mut self) -> alloc::vec::Vec<FanInfo> {
        let mut all_info = alloc::vec::Vec::new();
        
        for controller in &mut self.fan_controllers {
            if let Ok(info) = controller.get_fan_info() {
                all_info.push(info);
            }
        }
        
        // Guardar historial
        self.fan_history.extend(all_info.clone());
        
        // Mantener solo las últimas 1000 lecturas
        if self.fan_history.len() > 1000 {
            self.fan_history.drain(0..self.fan_history.len() - 1000);
        }
        
        all_info
    }

    /// Ajustar velocidad de fans basado en temperatura
    pub fn adjust_fans_by_temperature(&mut self, temperature: i32) {
        // Colectar las curvas primero para evitar conflictos de borrow
        let curves: alloc::vec::Vec<(FanLocation, FanCurve)> = self.fan_curves.clone();
        
        for controller in &mut self.fan_controllers {
            let location = controller.location;
            
            if let Some(curve) = curves.iter().find(|(loc, _)| loc == &location).map(|(_, c)| c) {
                let pwm = curve.calculate_pwm(temperature);
                let _ = controller.set_pwm_speed(pwm);
            }
        }
    }

    /// Ajustar velocidad de un fan específico
    pub fn set_fan_speed(&mut self, location: FanLocation, pwm: u8) -> Result<(), I2CError> {
        for controller in &mut self.fan_controllers {
            if controller.location == location {
                return controller.set_pwm_speed(pwm);
            }
        }
        Err(I2CError::DeviceNotResponding)
    }

    /// Obtener información de un fan específico
    pub fn get_fan_info(&mut self, location: FanLocation) -> Option<FanInfo> {
        for controller in &mut self.fan_controllers {
            if controller.location == location {
                if let Ok(info) = controller.get_fan_info() {
                    return Some(info);
                }
            }
        }
        None
    }

    /// Verificar si algún fan está fallando
    pub fn check_fan_failure(&self) -> alloc::vec::Vec<FanLocation> {
        let mut failed_fans = alloc::vec::Vec::new();
        
        for info in &self.fan_history {
            if info.state == FanState::Error {
                failed_fans.push(info.location);
            }
        }
        
        failed_fans
    }

    /// Obtener el RPM promedio de todos los fans
    pub fn get_average_rpm(&self) -> Option<u32> {
        if self.fan_history.is_empty() {
            return None;
        }
        
        let sum: u32 = self.fan_history.iter()
            .filter(|info| info.state == FanState::Running)
            .map(|info| info.rpm)
            .sum();
        
        let count = self.fan_history.iter()
            .filter(|info| info.state == FanState::Running)
            .count();
        
        if count > 0 {
            Some(sum / count as u32)
        } else {
            None
        }
    }
}

impl Default for FanControllerManager {
    fn default() -> Self {
        Self::new()
    }
}
