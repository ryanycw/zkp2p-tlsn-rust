use serde::Deserialize;
use std::fmt;

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NotaryConfig {
    pub server: ServerConfig,
    pub tls_enabled: bool,
}

#[derive(Debug, Clone)]
pub enum ProviderType {
    Wise,
    PayPal,
}

impl fmt::Display for ProviderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProviderType::Wise => write!(f, "wise"),
            ProviderType::PayPal => write!(f, "paypal"),
        }
    }
}
