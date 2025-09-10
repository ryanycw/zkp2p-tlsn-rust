use serde::Serialize;

use crate::domain::Provider;

pub fn get_file_path(provider: &str, content_type: &str) -> String {
    format!("{}.{}.tlsn", provider, content_type)
}

pub async fn save_file<T: Serialize>(
    provider: &Provider,
    content_type: &str,
    content: &T,
) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_file_path(&provider.to_string(), content_type);
    tokio::fs::write(&path, bincode::serialize(content)?).await?;
    println!("💾 Saved {} file to: {}", content_type, path);
    Ok(())
}
