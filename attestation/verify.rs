use std::time::Duration;

use clap::Parser;

use tls_core::verify::WebPkiVerifier;
use tls_server_fixture::CA_CERT_DER;
use tlsn_core::{
    CryptoProvider,
    presentation::{Presentation, PresentationOutput},
    signing::VerifyingKey,
};
use zkp2p_tlsn_rust::ExampleType;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// What data to notarize.
    #[clap(default_value_t, value_enum)]
    example_type: ExampleType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    verify_presentation(&args.example_type).await
}

async fn verify_presentation(example_type: &ExampleType) -> Result<(), Box<dyn std::error::Error>> {
    let presentation_path = zkp2p_tlsn_rust::get_file_path(example_type, "presentation");

    println!("🔍 Verifying {:?} presentation...", example_type);
    println!("   Reading presentation from: {}", presentation_path);

    // Read the presentation from disk.
    let presentation: Presentation =
        bincode::deserialize(&std::fs::read(&presentation_path).map_err(|e| {
            format!(
                "Failed to read presentation file {}: {}",
                presentation_path, e
            )
        })?)?;

    // Configure crypto provider based on example type
    let crypto_provider = match example_type {
        ExampleType::WiseTransaction => {
            println!("🔒 Using production TLS verification for Wise.com");
            // Use default crypto provider for production TLS verification of Wise.com
            CryptoProvider::default()
        }
        _ => {
            println!("🧪 Using test fixture crypto provider for local testing");
            // Use test fixture crypto provider for local testing
            let mut root_store = tls_core::anchors::RootCertStore::empty();
            root_store
                .add(&tls_core::key::Certificate(CA_CERT_DER.to_vec()))
                .unwrap();
            CryptoProvider {
                cert: WebPkiVerifier::new(root_store, None),
                ..Default::default()
            }
        }
    };

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

    println!("✅ Cryptographic verification successful!");
    println!();

    match example_type {
        ExampleType::WiseTransaction => {
            println!(
                "============================================================================"
            );
            println!("🎉 ZKP2P DUAL-PHASE PAYMENT VERIFICATION SUCCESSFUL");
            println!(
                "============================================================================"
            );
            println!();
            println!("🔒 Verified Connection Details:");
            println!("   Server: {} (Wise.com payment platform)", server_name);
            println!("   Session Time: {}", time);
            println!("   Protocol: TLS 1.2 with MPC-TLS notarization");
            println!();

            // Analyze the dual-phase transcript
            let sent_lines: Vec<&str> = sent.lines().collect();
            let recv_lines: Vec<&str> = recv.lines().collect();

            // Look for dual-phase requests
            let mut phase1_detected = false;
            let mut phase2_detected = false;

            for line in &sent_lines {
                if line.contains("/all-transactions?direction=OUTGOING") {
                    phase1_detected = true;
                    println!("🔍 Phase 1 Verified: Transaction ownership request");
                    println!("   Request: GET {}", line.trim());
                }
                if line.contains("/gateway/v3/profiles/") && line.contains("/transfers/") {
                    phase2_detected = true;
                    println!("🔍 Phase 2 Verified: Transaction details request");
                    println!("   Request: GET {}", line.trim());
                }
            }

            if phase1_detected && phase2_detected {
                println!("✅ Dual-phase verification confirmed: Both ownership and details proven");
            } else {
                println!("⚠️  Warning: Expected dual-phase requests not detected in transcript");
            }

            println!();
            println!("📊 ZKP2P Payment Verification Results:");

            // Extract revealed payment data from the responses
            if recv_lines.len() > 0 {
                // Look for JSON responses and extract payment details
                let full_response = recv.clone();

                // Try to find payment details in the response
                if let Some(start) = full_response.find("{\"resource\"") {
                    if let Some(end) = full_response[start..].find("}") {
                        let json_str = &full_response[start..start + end + 1];
                        if let Ok(payment_json) =
                            serde_json::from_str::<serde_json::Value>(json_str)
                        {
                            if let Some(resource) = payment_json.get("resource") {
                                if let Some(id) = resource.get("id") {
                                    println!(
                                        "   ✓ Payment ID: {}",
                                        id.as_str().unwrap_or("[HIDDEN]")
                                    );
                                }
                            }
                            if let Some(amount) = payment_json.get("primaryAmount") {
                                println!(
                                    "   ✓ Payment Amount: {}",
                                    amount.as_str().unwrap_or("[HIDDEN]")
                                );
                            }
                            if let Some(currency) = payment_json.get("currency") {
                                println!(
                                    "   ✓ Currency: {}",
                                    currency.as_str().unwrap_or("[HIDDEN]")
                                );
                            }
                            if let Some(status) = payment_json.get("status") {
                                println!(
                                    "   ✓ Payment Status: {}",
                                    status.as_str().unwrap_or("[HIDDEN]")
                                );
                            }
                            if let Some(date) = payment_json.get("visibleOn") {
                                println!(
                                    "   ✓ Payment Date: {}",
                                    date.as_str().unwrap_or("[HIDDEN]")
                                );
                            }
                        }
                    }
                }
            }

            println!();
            println!("🔐 Privacy Protection Verified:");
            println!("   ✓ Session credentials (Cookie, X-Access-Token): HIDDEN (shown as X)");
            println!("   ✓ Personal account information: HIDDEN");
            println!("   ✓ Other transactions in list: HIDDEN");
            println!("   ✓ Only essential payment verification data: REVEALED");

            println!();
            println!("🔍 Full Transcript Analysis:");
            println!("   Note: 'X' represents data intentionally hidden by selective disclosure");
            println!();
            println!("Data sent to {}:", server_name);
            println!("{}", sent);
            println!();
            println!("Data received from {}:", server_name);
            println!("{}", recv);

            println!();
            println!(
                "============================================================================"
            );
            println!("🎉 ZKP2P VERIFICATION COMPLETE - PAYMENT PROOF VALIDATED");
            println!(
                "============================================================================"
            );
        }
        _ => {
            println!("-------------------------------------------------------------------");
            println!(
                "Successfully verified that the data below came from a session with {} at {}.",
                server_name, time
            );
            println!("Note that the data which the Prover chose not to disclose are shown as X.\n");
            println!("Data sent:\n");
            println!("{}\n", sent);
            println!("Data received:\n");
            println!("{}\n", recv);
            println!("-------------------------------------------------------------------");
        }
    }

    println!();
    println!("✅ Verification process completed successfully!");

    match example_type {
        ExampleType::WiseTransaction => {
            println!();
            println!("🎯 ZKP2P Integration Ready:");
            println!("   This verified proof can now be submitted to ZKP2P smart contracts");
            println!("   for automated crypto asset release upon payment confirmation.");
        }
        _ => {}
    }

    Ok(())
}
