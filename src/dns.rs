//! DNS Client Module
//! 
//! This module implements a DNS client for domain name resolution.

extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::vec;
use alloc::format;

/// Tipo de registro DNS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsRecordType {
    /// A (IPv4 address)
    A = 1,
    /// AAAA (IPv6 address)
    Aaaa = 28,
    /// CNAME (canonical name)
    Cname = 5,
    /// MX (mail exchange)
    Mx = 15,
    /// NS (name server)
    Ns = 2,
    /// PTR (pointer)
    Ptr = 12,
    /// SOA (start of authority)
    Soa = 6,
    /// TXT (text)
    Txt = 16,
}

impl DnsRecordType {
    /// Crear desde u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(DnsRecordType::A),
            28 => Some(DnsRecordType::Aaaa),
            5 => Some(DnsRecordType::Cname),
            15 => Some(DnsRecordType::Mx),
            2 => Some(DnsRecordType::Ns),
            12 => Some(DnsRecordType::Ptr),
            6 => Some(DnsRecordType::Soa),
            16 => Some(DnsRecordType::Txt),
            _ => None,
        }
    }

    /// Convertir a u16
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

/// Clase de DNS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsClass {
    /// IN (Internet)
    In = 1,
    /// CH (Chaos)
    Ch = 3,
    /// HS (Hesiod)
    Hs = 4,
}

impl DnsClass {
    /// Crear desde u16
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            1 => Some(DnsClass::In),
            3 => Some(DnsClass::Ch),
            4 => Some(DnsClass::Hs),
            _ => None,
        }
    }

    /// Convertir a u16
    pub fn to_u16(&self) -> u16 {
        *self as u16
    }
}

/// Registro DNS
#[derive(Debug, Clone)]
pub struct DnsRecord {
    /// Nombre del dominio
    pub name: String,
    /// Tipo de registro
    pub record_type: DnsRecordType,
    /// Clase
    pub class: DnsClass,
    /// TTL (time to live)
    pub ttl: u32,
    /// Datos del registro
    pub data: Vec<u8>,
}

impl DnsRecord {
    /// Crear nuevo registro
    pub fn new(name: String, record_type: DnsRecordType, class: DnsClass, ttl: u32, data: Vec<u8>) -> Self {
        Self {
            name,
            record_type,
            class,
            ttl,
            data,
        }
    }

    /// Obtener dirección IPv4 si es registro A
    pub fn get_ipv4(&self) -> Option<[u8; 4]> {
        if self.record_type == DnsRecordType::A && self.data.len() >= 4 {
            Some([self.data[0], self.data[1], self.data[2], self.data[3]])
        } else {
            None
        }
    }

    /// Obtener dirección IPv6 si es registro AAAA
    pub fn get_ipv6(&self) -> Option<[u8; 16]> {
        if self.record_type == DnsRecordType::Aaaa && self.data.len() >= 16 {
            let mut addr = [0u8; 16];
            addr.copy_from_slice(&self.data[..16]);
            Some(addr)
        } else {
            None
        }
    }
}

/// Pregunta DNS
#[derive(Debug, Clone)]
pub struct DnsQuestion {
    /// Nombre del dominio
    pub name: String,
    /// Tipo de registro
    pub record_type: DnsRecordType,
    /// Clase
    pub class: DnsClass,
}

impl DnsQuestion {
    /// Crear nueva pregunta
    pub fn new(name: String, record_type: DnsRecordType, class: DnsClass) -> Self {
        Self {
            name,
            record_type,
            class,
        }
    }
}

/// Tipo de mensaje DNS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DnsMessageType {
    /// Query
    Query = 0,
    /// Response
    Response = 1,
}

/// Mensaje DNS
#[derive(Debug, Clone)]
pub struct DnsMessage {
    /// ID de transacción
    pub transaction_id: u16,
    /// Tipo de mensaje
    pub message_type: DnsMessageType,
    /// Preguntas
    pub questions: Vec<DnsQuestion>,
    /// Respuestas
    pub answers: Vec<DnsRecord>,
    /// Autoridades
    pub authorities: Vec<DnsRecord>,
    /// Adicionales
    pub additionals: Vec<DnsRecord>,
}

impl DnsMessage {
    /// Crear nuevo mensaje
    pub fn new(transaction_id: u16, message_type: DnsMessageType) -> Self {
        Self {
            transaction_id,
            message_type,
            questions: Vec::new(),
            answers: Vec::new(),
            authorities: Vec::new(),
            additionals: Vec::new(),
        }
    }

    /// Agregar pregunta
    pub fn add_question(&mut self, question: DnsQuestion) {
        self.questions.push(question);
    }

    /// Agregar respuesta
    pub fn add_answer(&mut self, record: DnsRecord) {
        self.answers.push(record);
    }
}

/// Cliente DNS
pub struct DnsClient {
    /// Servidores DNS
    pub dns_servers: Vec<[u8; 4]>,
    /// Timeout en segundos
    pub timeout: u32,
    /// Número de reintentos
    pub retries: u32,
}

impl DnsClient {
    /// Crear nuevo cliente
    pub fn new() -> Self {
        Self {
            dns_servers: vec![[8, 8, 8, 8], [8, 8, 4, 4]], // Google DNS
            timeout: 5,
            retries: 3,
        }
    }

    /// Agregar servidor DNS
    pub fn add_dns_server(&mut self, server: [u8; 4]) {
        self.dns_servers.push(server);
    }

    /// Consultar DNS
    pub fn query(&self, name: &str, record_type: DnsRecordType) -> Result<Vec<DnsRecord>, String> {
        // En un sistema real, esto enviaría una consulta DNS
        // Para este ejemplo, retornamos un resultado simulado
        let mut records = Vec::new();
        
        if record_type == DnsRecordType::A {
            // Simular respuesta A
            let data = vec![192, 168, 1, 1];
            let record = DnsRecord::new(String::from(name), record_type, DnsClass::In, 3600, data);
            records.push(record);
        }
        
        Ok(records)
    }

    /// Resolver nombre de dominio a IPv4
    pub fn resolve_ipv4(&self, name: &str) -> Result<[u8; 4], String> {
        let records = self.query(name, DnsRecordType::A)?;
        
        for record in records {
            if let Some(ipv4) = record.get_ipv4() {
                return Ok(ipv4);
            }
        }
        
        Err(String::from("No A record found"))
    }

    /// Resolver nombre de dominio a IPv6
    pub fn resolve_ipv6(&self, name: &str) -> Result<[u8; 16], String> {
        let records = self.query(name, DnsRecordType::Aaaa)?;
        
        for record in records {
            if let Some(ipv6) = record.get_ipv6() {
                return Ok(ipv6);
            }
        }
        
        Err(String::from("No AAAA record found"))
    }

    /// Reverse DNS lookup
    pub fn reverse_lookup(&self, ip: [u8; 4]) -> Result<String, String> {
        // En un sistema real, esto haría un reverse DNS lookup
        // Para este ejemplo, retornamos un nombre simulado
        let name = format!("{}.{}.{}.{}.in-addr.arpa", ip[3], ip[2], ip[1], ip[0]);
        Ok(name)
    }

    /// Generar reporte de estado
    pub fn generate_status_report(&self) -> String {
        let mut report = String::from("DNS Client Status\n");
        report.push_str("=================\n\n");
        
        report.push_str(&format!("Timeout: {} seconds\n", self.timeout));
        report.push_str(&format!("Retries: {}\n\n", self.retries));
        
        report.push_str("DNS Servers:\n");
        for server in &self.dns_servers {
            report.push_str(&format!("  {}.{}.{}.{}\n", server[0], server[1], server[2], server[3]));
        }
        
        report
    }
}

impl Default for DnsClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Utilidades de DNS
pub struct DnsUtils;

impl DnsUtils {
    /// Validar nombre de dominio
    pub fn is_valid_domain(name: &str) -> bool {
        if name.is_empty() || name.len() > 253 {
            return false;
        }
        
        // Verificar caracteres válidos
        for c in name.chars() {
            if !c.is_alphanumeric() && c != '.' && c != '-' {
                return false;
            }
        }
        
        true
    }

    /// Normalizar nombre de dominio
    pub fn normalize_domain(name: &str) -> String {
        let mut result = name.to_lowercase();
        
        // Remover punto final si existe
        if result.ends_with('.') {
            result.pop();
        }
        
        result
    }
}
