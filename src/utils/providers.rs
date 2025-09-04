use crate::domain::{ProviderConfig, ServerConfig};
use crate::utils::tls;
use http_body_util::BodyExt;
use hyper::StatusCode;
use serde_json::Value;

pub async fn execute_transaction_request(
    request_sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::Empty<hyper::body::Bytes>,
    >,
    provider: &ProviderConfig,
    server: &ServerConfig,
    user_agent: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Executing transaction attestation request:");

    // Request specific transaction details
    println!(
        "   Requesting transaction details for ID: {}",
        provider.transaction_id
    );
    let transaction_uri = provider.transaction_endpoint()?;
    let request = tls::build_request(
        &transaction_uri,
        &server.host,
        &provider.auth_headers(),
        "Requesting specific transaction details for attestation",
        user_agent,
    )?;

    let response = request_sender.send_request(request).await?;

    if response.status() != StatusCode::OK {
        return Err(format!(
            "❌ Transaction request failed - Server returned: {}",
            response.status()
        )
        .into());
    }

    //Read and verify response
    let response_headers = response.headers().clone();
    let response_body = response.into_body().collect().await?.to_bytes();
    let response_text = String::from_utf8_lossy(&response_body);

    println!("🔍 Response headers: {:?}", response_headers);
    println!(
        "   ✅ Transaction details retrieved ({} bytes)",
        response_body.len()
    );

    //Verify it's valid JSON
    let _transaction_data: Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse transaction JSON: {}", e))?;

    println!("🔍 Transaction data: {:?}", _transaction_data);

    println!("✨ Transaction attestation completed - Ready for notarization");

    Ok(())
}
