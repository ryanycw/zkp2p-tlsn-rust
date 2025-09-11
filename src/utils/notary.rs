use notary_client::{Accepted, NotarizationRequest, NotaryClient};
use tracing::debug;

/// Requests notarization from the notary server
pub async fn request_notarization(
    client: &NotaryClient,
    max_sent: usize,
    max_recv: usize,
) -> Result<Accepted, Box<dyn std::error::Error>> {
    let request = NotarizationRequest::builder()
        .max_sent_data(max_sent)
        .max_recv_data(max_recv)
        .build()?;

    let accepted = client
        .request_notarization(request)
        .await
        .map_err(|e| format!("Failed to connect to Notary server: {}", e))?;

    debug!("Notary connection established (session: {})", accepted.id);

    Ok(accepted)
}
