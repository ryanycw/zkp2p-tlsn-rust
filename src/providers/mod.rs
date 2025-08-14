pub mod wise;

use std::env;

/// Server configuration for different providers
#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub server_name: &'static str,
}

impl ServerConfig {
    /// Creates server configuration for test fixtures
    pub fn test_fixture() -> Self {
        use tlsn_server_fixture::DEFAULT_FIXTURE_PORT;
        use tls_server_fixture::SERVER_DOMAIN;
        
        let host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".to_string());
        let port = env::var("SERVER_PORT")
            .map(|p| p.parse().expect("SERVER_PORT should be valid integer"))
            .unwrap_or(DEFAULT_FIXTURE_PORT);
        
        println!("🧪 Target server: {}:{} (test fixture)", host, port);
        
        Self {
            host,
            port,
            server_name: SERVER_DOMAIN,
        }
    }
}