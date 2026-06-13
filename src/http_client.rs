//! HTTP/HTTPS Client para CRONOS W-OS (Hive AI)
//!
//! Este módulo implementa un cliente HTTP/HTTPS para Hive AI,
//! permitiendo comunicación con servicios web externos

use core::fmt;
use alloc::collections::BTreeMap;
use alloc::string::{String, ToString};
use alloc::format;
use alloc::vec::Vec;
use crate::capability::{Capability, Cell, CapabilityId, invoke_capability, invoke_capability_mut};
use crate::graph_kernel::{GraphKernel, NodeId, NodeType, EdgeType};

/// Estado del módulo HTTP Client
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpClientState {
    /// No inicializado
    Uninitialized,
    /// Inicializado
    Initialized,
    /// Activo
    Active,
    /// Conectando
    Connecting,
    /// Error
    Error(String),
}

/// Método HTTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    /// GET
    Get,
    /// POST
    Post,
    /// PUT
    Put,
    /// DELETE
    Delete,
    /// PATCH
    Patch,
    /// HEAD
    Head,
    /// OPTIONS
    Options,
}

/// Versión HTTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpVersion {
    /// HTTP/1.0
    Http10,
    /// HTTP/1.1
    Http11,
    /// HTTP/2
    Http2,
}

/// Estado de respuesta HTTP
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpStatus {
    /// 200 OK
    Ok,
    /// 201 Created
    Created,
    /// 204 No Content
    NoContent,
    /// 301 Moved Permanently
    MovedPermanently,
    /// 302 Found
    Found,
    /// 400 Bad Request
    BadRequest,
    /// 401 Unauthorized
    Unauthorized,
    /// 403 Forbidden
    Forbidden,
    /// 404 Not Found
    NotFound,
    /// 500 Internal Server Error
    InternalServerError,
    /// 502 Bad Gateway
    BadGateway,
    /// 503 Service Unavailable
    ServiceUnavailable,
    /// Otro código
    Other(u16),
}

impl HttpStatus {
    pub fn from_code(code: u16) -> Self {
        match code {
            200 => HttpStatus::Ok,
            201 => HttpStatus::Created,
            204 => HttpStatus::NoContent,
            301 => HttpStatus::MovedPermanently,
            302 => HttpStatus::Found,
            400 => HttpStatus::BadRequest,
            401 => HttpStatus::Unauthorized,
            403 => HttpStatus::Forbidden,
            404 => HttpStatus::NotFound,
            500 => HttpStatus::InternalServerError,
            502 => HttpStatus::BadGateway,
            503 => HttpStatus::ServiceUnavailable,
            _ => HttpStatus::Other(code),
        }
    }

    pub fn code(&self) -> u16 {
        match self {
            HttpStatus::Ok => 200,
            HttpStatus::Created => 201,
            HttpStatus::NoContent => 204,
            HttpStatus::MovedPermanently => 301,
            HttpStatus::Found => 302,
            HttpStatus::BadRequest => 400,
            HttpStatus::Unauthorized => 401,
            HttpStatus::Forbidden => 403,
            HttpStatus::NotFound => 404,
            HttpStatus::InternalServerError => 500,
            HttpStatus::BadGateway => 502,
            HttpStatus::ServiceUnavailable => 503,
            HttpStatus::Other(code) => *code,
        }
    }
}

/// Configuración de solicitud HTTP
#[derive(Debug, Clone)]
pub struct HttpRequestConfig {
    /// ID único de la solicitud
    pub request_id: u64,
    /// Método HTTP
    pub method: HttpMethod,
    /// URL
    pub url: String,
    /// Headers
    pub headers: BTreeMap<String, String>,
    /// Body
    pub body: Option<Vec<u8>>,
    /// Timeout (segundos)
    pub timeout_seconds: u32,
    /// Seguir redirecciones
    pub follow_redirects: bool,
}

impl HttpRequestConfig {
    pub fn new(request_id: u64, method: HttpMethod, url: String) -> Self {
        Self {
            request_id,
            method,
            url,
            headers: BTreeMap::new(),
            body: None,
            timeout_seconds: 30,
            follow_redirects: true,
        }
    }

    pub fn with_header(mut self, key: String, value: String) -> Self {
        self.headers.insert(key, value);
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_timeout(mut self, timeout: u32) -> Self {
        self.timeout_seconds = timeout;
        self
    }
}

/// FASE 16: Cookie para persistencia de sesión
#[derive(Debug, Clone)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: String,
    pub path: String,
    pub expires: u64,
}

/// Respuesta HTTP
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// ID de la solicitud
    pub request_id: u64,
    /// Estado HTTP
    pub status: HttpStatus,
    /// Headers de respuesta
    pub headers: BTreeMap<String, String>,
    /// Body de respuesta
    pub body: Vec<u8>,
    /// Tiempo de respuesta (ms)
    pub response_time_ms: u64,
    /// FASE 16: Cookies recibidas en la respuesta
    pub cookies: Vec<Cookie>,
}

/// Cliente HTTP/HTTPS
pub struct HttpClient {
    /// ID único del cliente
    pub client_id: u64,
    /// FASE 16: Almacén de cookies para soberanía web (Gmail/Bancos)
    pub cookie_jar: BTreeMap<String, Vec<Cookie>>,
    /// Estado actual
    pub state: HttpClientState,
    /// Capability de este cliente
    pub capability_id: Option<CapabilityId>,
    /// Nodo en el graph kernel
    pub graph_node_id: Option<NodeId>,
    /// User-Agent (Soberanía Web - Perfilado dinámico)
    pub user_agent: String,
    /// Habilitar HTTPS
    pub enable_https: bool,
    /// Métricas del cliente
    pub metrics: HttpClientMetrics,
}

/// Métricas del cliente HTTP
#[derive(Debug, Clone)]
pub struct HttpClientMetrics {
    /// Número de solicitudes enviadas
    pub requests_sent: u64,
    /// Número de respuestas recibidas
    pub responses_received: u64,
    /// Bytes enviados
    pub bytes_sent: u64,
    /// Bytes recibidos
    pub bytes_received: u64,
    /// Tiempo total de respuesta (ms)
    pub total_response_time_ms: u64,
    /// Errores
    pub errors: u64,
}

impl Default for HttpClientMetrics {
    fn default() -> Self {
        Self {
            requests_sent: 0,
            responses_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            total_response_time_ms: 0,
            errors: 0,
        }
    }
}

impl HttpClient {
    pub fn new(client_id: u64) -> Self {
        Self {
            client_id,
            cookie_jar: BTreeMap::new(),
            state: HttpClientState::Uninitialized,
            capability_id: None,
            graph_node_id: None,
            user_agent: String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"), // Perfil Chrome Legítimo
            enable_https: true,
            metrics: HttpClientMetrics::default(),
        }
    }

    /// FASE 16: Autocompletar formulario (Cyber-Autonomy)
    pub fn autofill_form(&mut self, fields: BTreeMap<String, String>) -> Result<(), String> {
        // En un sistema real, inyectaría estos valores en el DOM de la página cargada
        for (field, _value) in fields {
            crate::serial_println!("🤖 Hive AI: Autocompletando campo '{}'...", field);
        }
        Ok(())
    }

    /// FASE 16: Establecer perfil de User-Agent para evitar bloqueos (Google/YouTube/Bancos)
    pub fn set_legitimate_profile(&mut self, profile: &str) {
        self.user_agent = match profile {
            "chrome" => String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
            "safari" => String::from("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Safari/605.1.15"),
            "firefox" => String::from("Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:121.0) Gecko/20100101 Firefox/121.0"),
            _ => String::from("CRONOS-Sovereign-Browser/2.1"),
        };
    }

    /// Inicializar el cliente en el graph kernel
    pub fn initialize(&mut self, graph_kernel: &GraphKernel) -> Result<(), String> {
        if self.state != HttpClientState::Uninitialized {
            return Err(format!("Cliente ya inicializado, estado actual: {:?}", self.state));
        }

        // Crear nodo en el graph kernel para este cliente
        let node_id = graph_kernel.create_node(
            NodeType::File,
            format!("http_client_{}", self.client_id),
        );
        self.graph_node_id = Some(node_id);

        self.state = HttpClientState::Initialized;
        Ok(())
    }

    /// Activar el cliente
    pub fn activate(&mut self) -> Result<(), String> {
        if self.state != HttpClientState::Initialized {
            return Err(format!("Cliente no está en estado Initialized, estado actual: {:?}", self.state));
        }

        self.state = HttpClientState::Active;
        Ok(())
    }

    /// Enviar solicitud HTTP
    pub fn send_request(&mut self, config: HttpRequestConfig) -> Result<HttpResponse, String> {
        if self.state != HttpClientState::Active {
            return Err(format!("Cliente no está activo, estado actual: {:?}", self.state));
        }

        self.state = HttpClientState::Connecting;

        // En un sistema real, esto enviaría una solicitud HTTP real
        // Por ahora, simulamos la respuesta
        let response = HttpResponse {
            request_id: config.request_id,
            status: HttpStatus::Ok,
            headers: {
                let mut headers = BTreeMap::new();
                headers.insert(String::from("Content-Type"), String::from("application/json"));
                headers.insert(String::from("Content-Length"), String::from("0"));
                headers
            },
            body: Vec::new(),
            response_time_ms: 500,
            cookies: Vec::new(),
        };

        self.metrics.requests_sent += 1;
        self.metrics.responses_received += 1;
        self.metrics.bytes_sent += config.body.as_ref().map(|b| b.len()).unwrap_or(0) as u64;
        self.metrics.bytes_received += response.body.len() as u64;
        self.metrics.total_response_time_ms += response.response_time_ms;

        self.state = HttpClientState::Active;
        Ok(response)
    }

    /// GET request
    pub fn get(&mut self, url: String) -> Result<HttpResponse, String> {
        let request_id = self.metrics.requests_sent + 1;
        let config = HttpRequestConfig::new(request_id, HttpMethod::Get, url);
        self.send_request(config)
    }

    /// POST request
    pub fn post(&mut self, url: String, body: Vec<u8>) -> Result<HttpResponse, String> {
        let request_id = self.metrics.requests_sent + 1;
        let config = HttpRequestConfig::new(request_id, HttpMethod::Post, url).with_body(body);
        self.send_request(config)
    }

    /// PUT request
    pub fn put(&mut self, url: String, body: Vec<u8>) -> Result<HttpResponse, String> {
        let request_id = self.metrics.requests_sent + 1;
        let config = HttpRequestConfig::new(request_id, HttpMethod::Put, url).with_body(body);
        self.send_request(config)
    }

    /// DELETE request
    pub fn delete(&mut self, url: String) -> Result<HttpResponse, String> {
        let request_id = self.metrics.requests_sent + 1;
        let config = HttpRequestConfig::new(request_id, HttpMethod::Delete, url);
        self.send_request(config)
    }

    /// Verificar si el cliente está activo
    pub fn is_active(&self) -> bool {
        self.state == HttpClientState::Active
    }

    /// Obtener el estado actual
    pub fn state(&self) -> &HttpClientState {
        &self.state
    }
}

/// Integración HTTP Client para CRONOS W-OS (Hive AI)
pub struct CronosHttpClientIntegration {
    /// Clientes registrados (keyed by client_id)
    pub clients: BTreeMap<u64, HttpClient>,
    /// Estado del módulo HTTP Client
    pub state: HttpClientState,
    /// Graph kernel capability
    pub graph_kernel: Option<Cell<GraphKernel>>,
    /// Capability del módulo HTTP Client
    pub capability_id: Option<CapabilityId>,
    /// Siguiente ID de cliente
    pub next_client_id: u64,
}

impl CronosHttpClientIntegration {
    pub fn new() -> Self {
        Self {
            clients: BTreeMap::new(),
            state: HttpClientState::Uninitialized,
            graph_kernel: None,
            capability_id: None,
            next_client_id: 1,
        }
    }

    /// Establecer el graph kernel
    pub fn set_graph_kernel(&mut self, graph_kernel: GraphKernel) {
        self.graph_kernel = Some(Cell::new(graph_kernel));
        self.state = HttpClientState::Initialized;
    }

    /// Crear un nuevo cliente
    pub fn create_client(&mut self, client_id: u64) -> Result<u64, String> {
        if self.state == HttpClientState::Uninitialized {
            return Err(String::from("HTTP Client no inicializado. Llamar a set_graph_kernel primero."));
        }

        let mut client = HttpClient::new(client_id);

        // Inicializar el cliente en el graph kernel
        if let Some(ref graph_kernel) = self.graph_kernel {
            let cap = graph_kernel.capability();
            let result = invoke_capability(&cap, |gk| {
                client.initialize(gk)
            });
            if let Some(Err(e)) = result {
                return Err(e);
            }
        }

        self.clients.insert(client_id, client);
        self.next_client_id = client_id + 1;

        Ok(client_id)
    }

    /// Crear un cliente con ID automático
    pub fn create_default_client(&mut self) -> Result<u64, String> {
        let client_id = self.next_client_id;
        self.create_client(client_id)
    }

    /// Obtener un cliente por ID
    pub fn get_client(&self, client_id: u64) -> Option<&HttpClient> {
        self.clients.get(&client_id)
    }

    /// Obtener un cliente mutable por ID
    pub fn get_client_mut(&mut self, client_id: u64) -> Option<&mut HttpClient> {
        self.clients.get_mut(&client_id)
    }

    /// Activar un cliente
    pub fn activate_client(&mut self, client_id: u64) -> Result<(), String> {
        if let Some(client) = self.get_client_mut(client_id) {
            client.activate()
        } else {
            Err(format!("Cliente con ID {} no encontrado", client_id))
        }
    }

    /// Enviar solicitud con un cliente
    pub fn send_request(&mut self, client_id: u64, config: HttpRequestConfig) -> Result<HttpResponse, String> {
        if let Some(client) = self.get_client_mut(client_id) {
            client.send_request(config)
        } else {
            Err(format!("Cliente con ID {} no encontrado", client_id))
        }
    }

    /// GET request con un cliente
    pub fn get(&mut self, client_id: u64, url: String) -> Result<HttpResponse, String> {
        if let Some(client) = self.get_client_mut(client_id) {
            client.get(url)
        } else {
            Err(format!("Cliente con ID {} no encontrado", client_id))
        }
    }

    /// POST request con un cliente
    pub fn post(&mut self, client_id: u64, url: String, body: Vec<u8>) -> Result<HttpResponse, String> {
        if let Some(client) = self.get_client_mut(client_id) {
            client.post(url, body)
        } else {
            Err(format!("Cliente con ID {} no encontrado", client_id))
        }
    }

    /// PUT request con un cliente
    pub fn put(&mut self, client_id: u64, url: String, body: Vec<u8>) -> Result<HttpResponse, String> {
        if let Some(client) = self.get_client_mut(client_id) {
            client.put(url, body)
        } else {
            Err(format!("Cliente con ID {} no encontrado", client_id))
        }
    }

    /// DELETE request con un cliente
    pub fn delete(&mut self, client_id: u64, url: String) -> Result<HttpResponse, String> {
        if let Some(client) = self.get_client_mut(client_id) {
            client.delete(url)
        } else {
            Err(format!("Cliente con ID {} no encontrado", client_id))
        }
    }

    /// Obtener número de clientes
    pub fn client_count(&self) -> usize {
        self.clients.len()
    }

    /// Obtener número de clientes activos
    pub fn active_client_count(&self) -> usize {
        self.clients.values().filter(|c| c.is_active()).count()
    }

    /// Listar todos los clientes
    pub fn list_clients(&self) -> Vec<&HttpClient> {
        self.clients.values().collect()
    }

    /// Verificar si HTTP está soportado
    pub fn is_http_supported(&self) -> bool {
        // En un sistema real, esto verificaría si hay conexión de red
        // Por ahora, asumimos que está soportado
        true
    }

    /// Obtener el estado del módulo HTTP Client
    pub fn state(&self) -> &HttpClientState {
        &self.state
    }

    /// Obtener métricas agregadas
    pub fn get_aggregated_metrics(&self) -> HttpClientMetrics {
        let mut total = HttpClientMetrics::default();
        for client in self.clients.values() {
            total.requests_sent += client.metrics.requests_sent;
            total.responses_received += client.metrics.responses_received;
            total.bytes_sent += client.metrics.bytes_sent;
            total.bytes_received += client.metrics.bytes_received;
            total.total_response_time_ms += client.metrics.total_response_time_ms;
            total.errors += client.metrics.errors;
        }
        total
    }
}

impl Default for CronosHttpClientIntegration {
    fn default() -> Self {
        Self::new()
    }
}

/// Errores de integración HTTP Client
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpClientIntegrationError {
    ClientNotFound,
    ClientAlreadyActive,
    ClientNotActive,
    InvalidUrl,
    ConnectionFailed,
    Timeout,
    SslError,
    RequestFailed,
}

impl fmt::Display for HttpClientIntegrationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpClientIntegrationError::ClientNotFound => write!(f, "Client not found"),
            HttpClientIntegrationError::ClientAlreadyActive => write!(f, "Client is already active"),
            HttpClientIntegrationError::ClientNotActive => write!(f, "Client is not active"),
            HttpClientIntegrationError::InvalidUrl => write!(f, "Invalid URL"),
            HttpClientIntegrationError::ConnectionFailed => write!(f, "Connection failed"),
            HttpClientIntegrationError::Timeout => write!(f, "Request timeout"),
            HttpClientIntegrationError::SslError => write!(f, "SSL/TLS error"),
            HttpClientIntegrationError::RequestFailed => write!(f, "Request failed"),
        }
    }
}
