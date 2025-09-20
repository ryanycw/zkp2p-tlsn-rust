use serde::Serialize;
use tracing::debug;

use crate::domain::Provider;

pub fn get_file_path(provider: &str, transaction_id: &str, content_type: &str) -> String {
    format!("{}_{}.{}.tlsn", provider, transaction_id, content_type)
}

pub async fn save_file<T: Serialize>(
    provider: &Provider,
    transaction_id: &str,
    content_type: &str,
    content: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_file_path(&provider.to_string(), transaction_id, content_type);
    tokio::fs::write(&path, bincode::serialize(content)?).await?;
    debug!("Saved {} to {}", content_type, path);
    Ok(())
}
