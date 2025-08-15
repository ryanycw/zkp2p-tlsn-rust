use super::ServerConfig;
use crate::http;
use http_body_util::BodyExt;
use hyper::StatusCode;
use serde_json::Value;
use std::env;

/// Wise-specific configuration
#[derive(Debug, Clone)]
pub struct WiseConfig {
    pub profile_id: String,
    pub transaction_id: String,
    pub cookie: String,
    pub access_token: String,
    pub server_config: ServerConfig,
}

impl WiseConfig {
    /// Creates Wise configuration from CLI arguments
    pub fn new(
        profile_id: String,
        transaction_id: String,
        cookie: String,
        access_token: String,
    ) -> Self {
        println!("🔐 Configuring ZKP2P Wise payment verification:");
        println!("   Profile ID: {}", profile_id);
        println!("   Target Payment ID: {}", transaction_id);
        println!("   Session credentials will remain private in all proofs");

        let server_config = ServerConfig::wise();

        Self {
            profile_id,
            transaction_id,
            cookie,
            access_token,
            server_config,
        }
    }

    /// Returns headers for Wise API authentication
    pub fn auth_headers(&self) -> Vec<(&str, &str)> {
        vec![
            ("Cookie", self.cookie.as_str()),
            ("X-Access-Token", self.access_token.as_str()),
        ]
    }

    /// Returns the transaction details endpoint for direct attestation
    pub fn transaction_endpoint(&self) -> String {
        format!(
            "/gateway/v3/profiles/{}/transfers/{}",
            self.profile_id, self.transaction_id
        )
    }
}

impl ServerConfig {
    /// Creates server configuration for Wise.com
    pub fn wise() -> Self {
        let host = env::var("WISE_HOST").unwrap_or("wise.com".to_string());
        let port = env::var("WISE_PORT")
            .map(|p| p.parse().expect("WISE_PORT should be valid integer"))
            .unwrap_or(443);

        println!("🌐 Target server: {} (production HTTPS)", host);

        Self {
            host,
            port,
            server_name: "wise.com",
        }
    }
}

/// Retrieves and attests specific transaction details
pub async fn execute_transaction_request(
    request_sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::Empty<hyper::body::Bytes>,
    >,
    wise_config: &WiseConfig,
    server_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Executing transaction attestation request:");

    // Request specific transaction details
    println!("   Requesting transaction details for ID: {}", wise_config.transaction_id);
    let transaction_uri = wise_config.transaction_endpoint();
    let request = http::build_request(
        &transaction_uri,
        server_name,
        &wise_config.auth_headers(),
        "Requesting specific transaction details for attestation",
    )?;

    let response = request_sender.send_request(request).await?;

    if response.status() != StatusCode::OK {
        return Err(format!(
            "❌ Transaction request failed - Server returned: {}",
            response.status()
        )
        .into());
    }

    // Read and verify response
    let response_body = response.into_body().collect().await?.to_bytes();
    let response_text = String::from_utf8_lossy(&response_body);

    println!(
        "   ✅ Transaction details retrieved ({} bytes)",
        response_body.len()
    );

    // Verify it's valid JSON
    let _transaction_data: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse transaction JSON: {}", e))?;

    println!("✨ Transaction attestation completed - Ready for notarization");

    Ok(())
}
