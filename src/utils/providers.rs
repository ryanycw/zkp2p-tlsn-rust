use crate::domain::{ProviderConfig, ServerConfig};
use crate::utils::tls;
use anyhow::{Context, Result};
use hyper::StatusCode;

pub async fn execute_transaction_request(
    request_sender: &mut hyper::client::conn::http1::SendRequest<
        http_body_util::Empty<hyper::body::Bytes>,
    >,
    provider: &ProviderConfig,
    server: &ServerConfig,
    user_agent: &str,
) -> Result<()> {
    let endpoint = provider
        .transaction_endpoint()
        .context("Failed to get transaction endpoint")?;
    let headers = provider.auth_headers();
    let request = tls::build_request(
        &endpoint,
        &server.host,
        &headers,
        "Requesting specific transaction details for attestation",
        user_agent,
    )
    .context("Failed to build request")?;

    request_sender
        .send_request(request)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send request: {e}"))
        .and_then(|response| {
            response
                .status()
                .eq(&StatusCode::OK)
                .then_some(())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "❌ Transaction request failed - Server returned: {}",
                        response.status()
                    )
                })
        })?;

    Ok(())
}
