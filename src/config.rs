use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

use crate::domain::{NotaryConfig, ProviderType, ServerConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub user_agent: String,
    pub max_sent_data: usize,
    pub max_recv_data: usize,
    pub paypal: ServerConfig,
    pub wise: ServerConfig,
    pub notary: NotaryConfig,
}

// println!(
//     "📋 Requesting notarization with data limits: sent={}KB, recv={}KB",
//     max_sent / 1024,
//     max_recv / 1024
// );

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let mut s = Config::builder()
            .add_source(File::with_name("config/default").required(false))
            .add_source(config::Environment::with_prefix("ZKP2P"));

        if let Ok(env) = env::var("ZKP2P_ENV") {
            s = s.add_source(File::with_name(&format!("config/{}", env)).required(false));
        }

        s.build()?.try_deserialize()
    }

    pub fn server_config(&self, provider_type: ProviderType) -> ServerConfig {
        let server_config = match provider_type {
            ProviderType::PayPal => {
                let host = self.paypal.host.clone();
                let port = self.paypal.port;

                ServerConfig { host, port }
            }
            ProviderType::Wise => {
                let host = self.wise.host.clone();
                let port = self.wise.port;

                ServerConfig { host, port }
            }
        };
        println!("< Target server: {} (production HTTPS)", server_config.host);
        server_config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config() {
        let app_config = AppConfig::new().unwrap();
        let server_config = app_config.server_config(ProviderType::Wise);
        assert_eq!(server_config.host, "wise.com");
        assert_eq!(server_config.port, 443);

        let server_config = app_config.server_config(ProviderType::PayPal);
        assert_eq!(server_config.host, "www.paypal.com");
        assert_eq!(server_config.port, 443);

        let notary_config = app_config.notary.clone();
        assert_eq!(notary_config.server.host, "127.0.0.1");
        assert_eq!(notary_config.server.port, 7047);
        assert_eq!(notary_config.tls_enabled, true);
    }
}
