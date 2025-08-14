use std::fmt;
use serde_json::Value;

pub mod attestation;
pub mod http;
pub mod notary;
pub mod providers;
pub mod tls;

// Maximum number of bytes that can be sent from prover to server.
// Increased for dual-phase requests (transaction list + transaction details)
pub const MAX_SENT_DATA: usize = 1 << 13; // 8KB (was 4KB)

// Maximum number of bytes that can be received by prover from server.
// Significantly increased for Wise transaction list responses which can be large
pub const MAX_RECV_DATA: usize = 1 << 18; // 256KB (was 64KB)

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum ExampleType {
    #[default]
    Json,
    Html,
    Authenticated,
    WiseTransaction,
}

impl fmt::Display for ExampleType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub fn get_file_path(example_type: &ExampleType, content_type: &str) -> String {
    let example_type = example_type.to_string().to_ascii_lowercase();
    format!("example-{}.{}.tlsn", example_type, content_type)
}

/// Utility function to verify transaction exists in transaction list
/// Used in dual-phase attestation to prove transaction ownership before revealing details
pub fn verify_transaction_in_list(transaction_list: &Value, target_transaction_id: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Wise transaction list format: { "meta": {...}, "data": [transactions...] }
    let transactions = transaction_list
        .get("data")
        .and_then(|data| data.as_array())
        .ok_or("Invalid transaction list format: missing 'data' array")?;

    for transaction in transactions {
        if let Some(id) = transaction.get("id").and_then(|id| id.as_str()) {
            if id == target_transaction_id {
                return Ok(true);
            }
        }
        // Also check nested resource.id for Wise API compatibility
        if let Some(resource) = transaction.get("resource") {
            if let Some(id) = resource.get("id").and_then(|id| id.as_str()) {
                if id == target_transaction_id {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Extract essential transaction metadata for logging (without exposing sensitive data)
pub fn extract_transaction_metadata(transaction: &Value) -> TransactionMetadata {
    TransactionMetadata {
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

/// Transaction metadata for logging and verification (excludes sensitive data)
#[derive(Debug)]
pub struct TransactionMetadata {
    pub id: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub date: String,
}
