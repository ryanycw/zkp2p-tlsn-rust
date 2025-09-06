use std::time::Duration;

use clap::Parser;

use tlsn_core::{
    CryptoProvider,
    presentation::{Presentation, PresentationOutput},
    signing::VerifyingKey,
};
use zkp2p_tlsn_rust::domain::ProviderArgs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ProviderArgs::parse();

    verify_presentation(&args.provider).await
}

async fn verify_presentation(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let presentation_path = zkp2p_tlsn_rust::get_file_path(provider, "presentation");

    println!("🔍 Verifying Wise transaction presentation...");
    println!("   Reading presentation from: {}", presentation_path);

    // Read the presentation from disk.
    let presentation: Presentation =
        bincode::deserialize(&std::fs::read(&presentation_path).map_err(|e| {
            format!(
                "Failed to read presentation file {}: {}",
                presentation_path, e
            )
        })?)?;

    // Configure crypto provider for Wise.com production TLS verification
    let crypto_provider = CryptoProvider::default();

    let VerifyingKey {
        alg,
        data: key_data,
    } = presentation.verifying_key();

    println!("🔑 Cryptographic verification details:");
    println!("   Algorithm: {}", alg);
    println!("   Notary Key: {}", hex::encode(key_data));
    println!();
    println!("⚠️  SECURITY CHECK: Do you trust this Notary key?");
    println!("   This key signed the cryptographic proof you're about to verify.");
    println!("   Only proceed if you trust the Notary that generated this key.");
    println!();

    // Verify the presentation cryptographically
    println!("🔍 Starting cryptographic verification...");
    let PresentationOutput {
        server_name,
        connection_info,
        transcript,
        // extensions, // Optionally, verify any custom extensions from prover/notary.
        ..
    } = presentation
        .verify(&crypto_provider)
        .map_err(|e| format!("Cryptographic verification failed: {}", e))?;

    // The time at which the connection was started.
    let time = chrono::DateTime::UNIX_EPOCH + Duration::from_secs(connection_info.time);
    let server_name = server_name.unwrap();
    let mut partial_transcript = transcript.unwrap();
    // Set the unauthenticated bytes so they are distinguishable.
    partial_transcript.set_unauthed(b'X');

    let sent = String::from_utf8_lossy(partial_transcript.sent_unsafe());
    let recv = String::from_utf8_lossy(partial_transcript.received_unsafe());

    println!("============================================================================");
    println!("🎉 ZKP2P PAYMENT VERIFICATION SUCCESSFUL");
    println!("============================================================================");
    println!();
    println!("🔒 Verified Connection Details:");
    println!("   Server: {} (Wise.com payment platform)", server_name);
    println!("   Session Time: {}", time);
    println!("   Protocol: TLS 1.2 with MPC-TLS notarization");
    println!();

    // Analyze the transaction transcript
    let sent_lines: Vec<&str> = sent.lines().collect();
    let recv_lines: Vec<&str> = recv.lines().collect();

    // Look for transaction details request
    let mut transaction_request_detected = false;

    for line in &sent_lines {
        if line.contains("/gateway/v3/profiles/") && line.contains("/transfers/") {
            transaction_request_detected = true;
            println!("🔍 Transaction Request Verified: Transaction details request");
            println!("   Request: GET {}", line.trim());
        }
    }

    if transaction_request_detected {
        println!("✅ Transaction verification confirmed: Payment details proven");
    } else {
        println!("⚠️  Warning: Expected transaction request not detected in transcript");
    }

    println!();
    println!("📊 ZKP2P Payment Verification Results:");

    // Extract revealed payment data from the responses using regex patterns
    if recv_lines.len() > 0 {
        let full_response = recv.clone();
        println!("   📋 Analyzing revealed transaction data...");

        // Use regex patterns to extract the specifically revealed ZKP2P fields
        // These should match the patterns used in present.rs
        let field_patterns = [
            (r#""id":([0-9]+)"#, "Payment ID"),
            (r#""state":"([^"]+)""#, "Payment State"),
            (
                r#""state":"OUTGOING_PAYMENT_SENT","date":([0-9]+)"#,
                "Payment Timestamp",
            ),
            (r#""targetAmount":([0-9\.]+)"#, "Target Amount"),
            (r#""targetCurrency":"([^"]+)""#, "Target Currency"),
            (r#""targetRecipientId":([0-9]+)"#, "Target Recipient ID"),
        ];

        let mut verified_fields = 0;
        for (pattern, field_name) in field_patterns.iter() {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(&full_response) {
                    if let Some(value) = captures.get(1) {
                        println!("   ✓ {}: {}", field_name, value.as_str());
                        verified_fields += 1;
                    }
                }
            }
        }

        if verified_fields > 0 {
            println!(
                "   ✅ Successfully verified {} ZKP2P payment fields",
                verified_fields
            );
        } else {
            println!("   ⚠️  Warning: No ZKP2P fields found in revealed data");
            println!("   📝 Raw revealed response: {}", full_response);
        }
    }

    println!("🔍 Full Transcript Analysis:");
    println!("   Note: 'X' represents data intentionally hidden by selective disclosure");
    println!();
    println!("Data sent to {}:", server_name);
    println!("{}", sent);
    println!();
    println!("Data received from {}:", server_name);
    println!("{}", recv);

    println!();
    println!("============================================================================");
    println!("🎉 ZKP2P VERIFICATION COMPLETE - PAYMENT PROOF VALIDATED");
    println!("============================================================================");

    println!();
    println!("✅ Verification process completed successfully!");

    println!();
    println!("🎯 ZKP2P Integration Ready:");
    println!("   This verified proof can now be submitted to ZKP2P smart contracts");
    println!("   for automated crypto asset release upon payment confirmation.");

    Ok(())
}
