use crate::ExampleType;
use notary_client::{Accepted, NotarizationRequest, NotaryClient};
use std::env;

/// Configuration for notary connection
#[derive(Debug, Clone)]
pub struct NotaryConfig {
    pub host: String,
    pub port: u16,
    pub tls_enabled: bool,
}

impl NotaryConfig {
    /// Creates notary configuration from environment variables
    pub fn from_env(example_type: &ExampleType) -> Self {
        let host = env::var("NOTARY_HOST").unwrap_or("127.0.0.1".to_string());
        let port = env::var("NOTARY_PORT")
            .map(|p| p.parse().expect("NOTARY_PORT should be valid integer"))
            .unwrap_or(7047);

        let tls_enabled = match example_type {
            ExampleType::WiseTransaction => {
                let notary_tls_raw = env::var("NOTARY_TLS").unwrap_or("true".to_string());
                println!(
                    "🔍 Debug: NOTARY_TLS environment variable = '{}'",
                    notary_tls_raw
                );
                let tls = notary_tls_raw.parse().unwrap_or(true);
                println!(
                    "🔒 Notary TLS: {}",
                    if tls {
                        "enabled (production)"
                    } else {
                        "disabled (testing)"
                    }
                );
                tls
            }
            _ => {
                println!("🔓 Notary TLS: disabled (test fixtures)");
                false
            }
        };

        Self {
            host,
            port,
            tls_enabled,
        }
    }

    /// Builds a notary client from the configuration
    pub fn build_client(&self) -> NotaryClient {
        println!(
            "📡 Connecting to Notary server: {}:{}",
            self.host, self.port
        );

        NotaryClient::builder()
            .host(&self.host)
            .port(self.port)
            .enable_tls(self.tls_enabled)
            .build()
            .unwrap()
    }
}

/// Requests notarization from the notary server
pub async fn request_notarization(
    client: &NotaryClient,
    max_sent: usize,
    max_recv: usize,
) -> Result<Accepted, Box<dyn std::error::Error>> {
    println!(
        "📋 Requesting notarization with data limits: sent={}KB, recv={}KB",
        max_sent / 1024,
        max_recv / 1024
    );

    let request = NotarizationRequest::builder()
        .max_sent_data(max_sent)
        .max_recv_data(max_recv)
        .build()?;

    let accepted = client
        .request_notarization(request)
        .await
        .expect("❌ Failed to connect to Notary server. Ensure it's running and accessible.");

    println!(
        "✅ Notary connection established (session: {})",
        accepted.id
    );

    Ok(accepted)
}
