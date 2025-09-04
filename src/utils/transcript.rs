use spansy::Spanned;
use tlsn_formats::http::HttpTranscript;
use tracing::debug;

use crate::domain::ProviderType;

/// Analyzes and logs Wise transaction transcript data
pub fn analyze_transcript(transcript: &HttpTranscript) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Analyzing dual-phase transcript...");

    if transcript.responses.len() >= 2 {
        // Phase 1: Transaction list response
        let list_body = &transcript.responses[0].body.as_ref().unwrap().content;
        let list_data = String::from_utf8_lossy(list_body.span().as_bytes());
        println!(
            "   Phase 1 Response: Transaction list ({} bytes)",
            list_data.len()
        );

        // Phase 2: Transaction details response
        let details_body = &transcript.responses[1].body.as_ref().unwrap().content;
        let details_data = String::from_utf8_lossy(details_body.span().as_bytes());
        println!(
            "   Phase 2 Response: Transaction details ({} bytes)",
            details_data.len()
        );

        // Extract and log transaction metadata (without sensitive data)
        if let Ok(transaction_json) = serde_json::from_str::<serde_json::Value>(&details_data) {
            let metadata = crate::extract_transaction_metadata(&transaction_json);
            println!("   📊 Transaction Summary:");
            println!("      ID: {}", metadata.id);
            println!("      Amount: {} {}", metadata.amount, metadata.currency);
            println!("      Status: {}", metadata.status);
            println!("      Date: {}", metadata.date);
            debug!(
                "Full transaction data: {}",
                serde_json::to_string_pretty(&transaction_json)?
            );
        }
    } else {
        return Err("Dual-phase transcript should contain at least 2 responses".into());
    }

    Ok(())
}

pub async fn save_attestation_files(
    provider: &ProviderType,
    attestation: &tlsn_core::attestation::Attestation,
    secrets: &tlsn_core::Secrets,
) -> Result<(), Box<dyn std::error::Error>> {
    let attestation_path = crate::get_file_path(&provider.to_string(), "attestation");
    let secrets_path = crate::get_file_path(&provider.to_string(), "secrets");

    println!("💾 Saving attestation files...");
    tokio::fs::write(&attestation_path, bincode::serialize(attestation)?).await?;
    tokio::fs::write(&secrets_path, bincode::serialize(secrets)?).await?;

    println!("✨ ZKP2P dual-phase payment proof process completed successfully!");
    println!();
    println!("📁 Generated files:");
    println!(
        "   🔒 Dual-Phase Proof: {} (cryptographic attestation signed by Notary)",
        attestation_path
    );
    println!(
        "   🔑 Selective Disclosure Data: {} (for privacy-preserving presentation)",
        secrets_path
    );
    println!();
    println!("🎯 Next steps for ZKP2P payment verification:");
    println!("   1. Create selective proof: cargo run --release --bin zkp2p-present");
    println!("   2. Verify payment proof: cargo run --release --bin zkp2p-verify");
    println!("   3. Submit to ZKP2P protocol for crypto asset release");
    Ok(())
}
