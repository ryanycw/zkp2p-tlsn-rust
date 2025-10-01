use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::env;

use crate::domain::{NotaryConfig, ServerConfig};

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub user_agent: String,
    pub max_sent_data: usize,
    pub max_recv_data: usize,
    pub paypal: ServerConfig,
    pub wise: ServerConfig,
    pub notary: NotaryConfig,
    pub unauthed_bytes: String,
}

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

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let app_config = AppConfig::new().unwrap();
        assert_eq!(app_config.wise.host, "wise.com");
        assert_eq!(app_config.wise.port, 443);

        assert_eq!(app_config.paypal.host, "www.paypal.com");
        assert_eq!(app_config.paypal.port, 443);

        let notary_config = app_config.notary.clone();
        assert_eq!(notary_config.server.host, "127.0.0.1");
        assert_eq!(notary_config.server.port, 7047);
        assert_eq!(notary_config.tls_enabled, false);
    }
}
