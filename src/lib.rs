use serde_json::Value;

pub mod config;
pub mod domain;
pub mod utils;

pub fn get_file_path(provider: &str, content_type: &str) -> String {
    format!("{}.{}.tlsn", provider, content_type)
}

pub fn extract_transaction_metadata(transaction: &Value) -> domain::TransactionMetadata {
    domain::TransactionMetadata {
        id: transaction
            .get("resource")
            .and_then(|r| r.get("id"))
            .and_then(|id| id.as_str())
            .unwrap_or("unknown")
            .to_string(),
        amount: transaction
            .get("primaryAmount")
            .and_then(|a| a.as_str())
            .unwrap_or("unknown")
            .to_string(),
        currency: transaction
            .get("currency")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown")
            .to_string(),
        status: transaction
            .get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("unknown")
            .to_string(),
        date: transaction
            .get("visibleOn")
            .and_then(|d| d.as_str())
            .unwrap_or("unknown")
            .to_string(),
    }
}
