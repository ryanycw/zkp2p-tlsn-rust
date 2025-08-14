use std::env;
use http_body_util::BodyExt;
use hyper::StatusCode;
use serde_json::Value;
use crate::http;
use super::ServerConfig;

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
        println!("🔐 Configuring ZKP2P Wise dual-phase payment verification:");
        println!("   Profile ID: {}", profile_id);
        println!("   Target Payment ID: {}", transaction_id);
        println!("   Phase 1: Verify transaction ownership");
        println!("   Phase 2: Attest transaction details");
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
    
    /// Returns the transaction list endpoint
    pub fn transaction_list_endpoint() -> &'static str {
        "/all-transactions?direction=OUTGOING"
    }
    
    /// Returns the transaction details endpoint
    pub fn transaction_details_endpoint(&self) -> String {
        format!("/gateway/v3/profiles/{}/transfers/{}", 
                self.profile_id, self.transaction_id)
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

/// Verifies transaction ownership and retrieves details in dual-phase
pub async fn execute_dual_phase_requests(
    request_sender: &mut hyper::client::conn::http1::SendRequest<http_body_util::Empty<hyper::body::Bytes>>,
    wise_config: &WiseConfig,
    server_name: &str,
) -> Result<(), Box<dyn std::error::Error>>
{
    println!("🔄 Executing dual-phase MPC-TLS requests:");
    
    // Phase 1: Transaction List Verification Request
    println!("   Phase 1: Verifying transaction ownership...");
    let phase1_request = http::build_request(
        WiseConfig::transaction_list_endpoint(),
        server_name,
        &wise_config.auth_headers(),
        "Requesting transaction list for ownership verification"
    )?;
    
    let phase1_response = request_sender.send_request(phase1_request).await?;
    
    if phase1_response.status() != StatusCode::OK {
        return Err(format!("❌ Phase 1 failed - Transaction list request returned: {}", 
                          phase1_response.status()).into());
    }
    
    // Read and parse transaction list response
    let phase1_body = phase1_response.into_body().collect().await?.to_bytes();
    let phase1_text = String::from_utf8_lossy(&phase1_body);
    
    println!("   ✅ Transaction list retrieved ({} bytes)", phase1_body.len());
    
    // Verify target transaction exists in user's transaction list
    let transaction_list: Value = serde_json::from_str(&phase1_text)
        .map_err(|e| format!("Failed to parse transaction list JSON: {}", e))?;
    
    let transaction_exists = crate::verify_transaction_in_list(
        &transaction_list,
        &wise_config.transaction_id
    )?;
    
    if !transaction_exists {
        return Err(format!("❌ Transaction {} not found in user's transaction list - cannot prove ownership", 
                          wise_config.transaction_id).into());
    }
    
    println!("   ✅ Transaction ownership verified: {} found in user's list", 
             wise_config.transaction_id);
    
    // Phase 2: Transaction Details Attestation Request
    println!("   Phase 2: Attesting transaction details...");
    let phase2_uri = wise_config.transaction_details_endpoint();
    let phase2_request = http::build_request(
        &phase2_uri,
        server_name,
        &wise_config.auth_headers(),
        "Requesting specific transaction details for attestation"
    )?;
    
    let phase2_response = request_sender.send_request(phase2_request).await?;
    
    if phase2_response.status() != StatusCode::OK {
        return Err(format!("❌ Phase 2 failed - Transaction details request returned: {}", 
                          phase2_response.status()).into());
    }
    
    println!("   ✅ Transaction details retrieved successfully");
    println!("✨ Dual-phase verification completed - Ready for attestation");
    
    Ok(())
}