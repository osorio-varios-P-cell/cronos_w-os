//! Unit Tests Framework Module
//! 
//! This module implements a unit testing framework for the kernel that provides
//! basic assertion macros, test runner infrastructure, and test result reporting
//! suitable for a no_std environment.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Resultado de un test
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestResult {
    /// Test pasó
    Passed,
    /// Test falló
    Failed,
    /// Test fue ignorado
    Ignored,
}

/// Información de un test
#[derive(Debug, Clone)]
pub struct TestInfo {
    /// Nombre del test
    pub name: String,
    /// Nombre del módulo
    pub module: String,
    /// Resultado
    pub result: TestResult,
    /// Mensaje de error (si falló)
    pub error_message: Option<String>,
    /// Duración del test en milisegundos
    pub duration_ms: u64,
}

impl TestInfo {
    /// Crear nueva información de test
    pub fn new(name: String, module: String) -> Self {
        Self {
            name,
            module,
            result: TestResult::Passed,
            error_message: None,
            duration_ms: 0,
        }
    }

    /// Marcar el test como fallido
    pub fn fail(&mut self, message: String) {
        self.result = TestResult::Failed;
        self.error_message = Some(message);
    }

    /// Marcar el test como ignorado
    pub fn ignore(&mut self) {
        self.result = TestResult::Ignored;
    }

    /// Verificar si el test pasó
    pub fn passed(&self) -> bool {
        self.result == TestResult::Passed
    }

    /// Verificar si el test falló
    pub fn failed(&self) -> bool {
        self.result == TestResult::Failed
    }
}

/// Resumen de ejecución de tests
#[derive(Debug, Clone)]
pub struct TestSummary {
    /// Total de tests
    pub total: usize,
    /// Tests que pasaron
    pub passed: usize,
    /// Tests que fallaron
    pub failed: usize,
    /// Tests ignorados
    pub ignored: usize,
    /// Duración total en milisegundos
    pub total_duration_ms: u64,
}

impl TestSummary {
    /// Crear nuevo resumen
    pub fn new() -> Self {
        Self {
            total: 0,
            passed: 0,
            failed: 0,
            ignored: 0,
            total_duration_ms: 0,
        }
    }

    /// Calcular el porcentaje de tests que pasaron
    pub fn pass_percentage(&self) -> f32 {
        if self.total == 0 {
            100.0
        } else {
            (self.passed as f32 / self.total as f32) * 100.0
        }
    }

    /// Verificar si todos los tests pasaron
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }
}

impl Default for TestSummary {
    fn default() -> Self {
        Self::new()
    }
}

/// Función de test
pub type TestFunction = fn() -> Result<(), String>;

/// Registro de test
pub struct TestRegistry {
    /// Tests registrados
    tests: Vec<(String, String, TestFunction)>,
}

impl TestRegistry {
    /// Crear nuevo registro
    pub fn new() -> Self {
        Self {
            tests: Vec::new(),
        }
    }

    /// Registrar un test
    pub fn register(&mut self, module: String, name: String, test_fn: TestFunction) {
        self.tests.push((module, name, test_fn));
    }

    /// Ejecutar todos los tests registrados
    pub fn run_all(&self) -> (Vec<TestInfo>, TestSummary) {
        let mut results = Vec::new();
        let mut summary = TestSummary::new();

        for (module, name, test_fn) in &self.tests {
            let mut test_info = TestInfo::new(name.clone(), module.clone());
            
            // En un sistema real, medir el tiempo aquí
            match test_fn() {
                Ok(_) => {
                    test_info.result = TestResult::Passed;
                    summary.passed += 1;
                }
                Err(msg) => {
                    test_info.fail(msg);
                    summary.failed += 1;
                }
            }

            summary.total += 1;
            results.push(test_info);
        }

        (results, summary)
    }

    /// Obtener el número de tests registrados
    pub fn test_count(&self) -> usize {
        self.tests.len()
    }
}

impl Default for TestRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Macro de aserción para igualdad
#[macro_export]
macro_rules! assert_eq {
    ($left:expr, $right:expr) => {
        if $left != $right {
            return Err(alloc::format!(
                "assertion failed: `{:?}` != `{:?}`",
                $left, $right
            ));
        }
    };
    ($left:expr, $right:expr, $($arg:tt)*) => {
        if $left != $right {
            return Err(alloc::format!(
                "assertion failed: `{:?}` != `{:?}`: {}",
                $left, $right, alloc::format!($($arg)*)
            ));
        }
    };
}

/// Macro de aserción para desigualdad
#[macro_export]
macro_rules! assert_ne {
    ($left:expr, $right:expr) => {
        if $left == $right {
            return Err(alloc::format!(
                "assertion failed: `{:?}` == `{:?}`",
                $left, $right
            ));
        }
    };
    ($left:expr, $right:expr, $($arg:tt)*) => {
        if $left == $right {
            return Err(alloc::format!(
                "assertion failed: `{:?}` == `{:?}`: {}",
                $left, $right, alloc::format!($($arg)*)
            ));
        }
    };
}

/// Macro de aserción para verdad
#[macro_export]
macro_rules! assert {
    ($cond:expr) => {
        if !$cond {
            return Err(alloc::format!(
                "assertion failed: `{}`",
                stringify!($cond)
            ));
        }
    };
    ($cond:expr, $($arg:tt)*) => {
        if !$cond {
            return Err(alloc::format!(
                "assertion failed: `{}`: {}",
                stringify!($cond), alloc::format!($($arg)*)
            ));
        }
    };
}

/// Macro para registrar un test
#[macro_export]
macro_rules! test {
    ($registry:expr, $module:expr, $name:expr, $test_fn:expr) => {
        $registry.register($module.to_string(), $name.to_string(), $test_fn);
    };
}

/// Gestor de tests global
pub struct TestManager {
    /// Registro de tests
    registry: TestRegistry,
    /// Resultados de la última ejecución
    last_results: Vec<TestInfo>,
    /// Resumen de la última ejecución
    last_summary: TestSummary,
}

impl TestManager {
    /// Crear nuevo gestor de tests
    pub fn new() -> Self {
        Self {
            registry: TestRegistry::new(),
            last_results: Vec::new(),
            last_summary: TestSummary::new(),
        }
    }

    /// Registrar un test
    pub fn register_test(&mut self, module: String, name: String, test_fn: TestFunction) {
        self.registry.register(module, name, test_fn);
    }

    /// Ejecutar todos los tests
    pub fn run_tests(&mut self) -> &TestSummary {
        let (results, summary) = self.registry.run_all();
        self.last_results = results;
        self.last_summary = summary;
        &self.last_summary
    }

    /// Obtener los resultados de la última ejecución
    pub fn get_results(&self) -> &Vec<TestInfo> {
        &self.last_results
    }

    /// Obtener el resumen de la última ejecución
    pub fn get_summary(&self) -> &TestSummary {
        &self.last_summary
    }

    /// Obtener tests fallidos
    pub fn get_failed_tests(&self) -> Vec<&TestInfo> {
        self.last_results.iter()
            .filter(|t| t.failed())
            .collect()
    }

    /// Generar reporte de texto
    pub fn generate_report(&self) -> String {
        let mut report = String::from("Test Report\n");
        report.push_str("===========\n\n");

        report.push_str(&format!(
            "Total: {}, Passed: {}, Failed: {}, Ignored: {}\n",
            self.last_summary.total,
            self.last_summary.passed,
            self.last_summary.failed,
            self.last_summary.ignored
        ));

        report.push_str(&format!(
            "Pass Rate: {:.1}%\n\n",
            self.last_summary.pass_percentage()
        ));

        if self.last_summary.failed > 0 {
            report.push_str("Failed Tests:\n");
            report.push_str("-------------\n");
            for test in self.get_failed_tests() {
                report.push_str(&format!(
                    "  [{}] {}::{}\n",
                    test.module, test.name,
                    if let Some(ref msg) = test.error_message {
                        format!(" - {}", msg)
                    } else {
                        String::new()
                    }
                ));
            }
            report.push('\n');
        }

        report
    }
}

impl Default for TestManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Tests de ejemplo para el framework
#[cfg(test)]
mod example_tests {
    use super::*;

    #[test]
    fn test_assert_eq() {
        assert_eq!(1, 1);
        assert_eq!(2, 2);
    }

    #[test]
    fn test_assert_ne() {
        assert_ne!(1, 2);
        assert_ne!(3, 4);
    }

    #[test]
    fn test_assert() {
        assert!(true);
        assert!(1 == 1);
    }

    #[test]
    fn test_math_operations() {
        let a = 5;
        let b = 3;
        assert_eq!(a + b, 8);
        assert_eq!(a - b, 2);
        assert_eq!(a * b, 15);
        assert_eq!(a / b, 1);
    }
}
