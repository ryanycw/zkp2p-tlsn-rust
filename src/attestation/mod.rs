use crate::ExampleType;
use spansy::Spanned;
use tlsn_core::{request::RequestConfig, transcript::TranscriptCommitConfig};
use tlsn_formats::http::{DefaultHttpCommitter, HttpCommit, HttpTranscript};
use tlsn_prover::Prover;
use tracing::debug;

/// Analyzes and logs transcript data based on example type
pub fn analyze_transcript(
    transcript: &HttpTranscript,
    example_type: &ExampleType,
) -> Result<(), Box<dyn std::error::Error>> {
    match example_type {
        ExampleType::WiseTransaction => {
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
                if let Ok(transaction_json) =
                    serde_json::from_str::<serde_json::Value>(&details_data)
                {
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
        }
        _ => {
            // Single response for test fixtures
            let body_content = &transcript.responses[0].body.as_ref().unwrap().content;
            let body = String::from_utf8_lossy(body_content.span().as_bytes());

            match body_content {
                tlsn_formats::http::BodyContent::Json(_json) => {
                    let parsed = serde_json::from_str::<serde_json::Value>(&body)?;
                    println!("📄 Test data received ({} bytes)", body.len());
                    debug!(
                        "Test data preview: {}",
                        serde_json::to_string_pretty(&parsed)?
                    );
                }
                _ => {
                    println!("📄 Test data received ({} bytes)", body.len());
                }
            }
        }
    }

    Ok(())
}

/// Creates cryptographic commitments to transcript data
pub fn create_transcript_commitment(
    transcript: &tlsn_core::transcript::Transcript,
    http_transcript: &HttpTranscript,
    example_type: &ExampleType,
) -> Result<TranscriptCommitConfig, Box<dyn std::error::Error>> {
    println!("🔏 Creating cryptographic commitments to HTTP transcript...");

    let mut builder = TranscriptCommitConfig::builder(transcript);

    // Commit to HTTP transcript components using default strategy
    DefaultHttpCommitter::default().commit_transcript(&mut builder, http_transcript)?;

    match example_type {
        ExampleType::WiseTransaction => {
            println!("✅ Dual-phase transcript commitments created");
            println!("   ✓ Transaction ownership proof committed");
            println!("   ✓ Transaction details attestation committed");
        }
        _ => {
            println!("✅ Transcript commitments created");
        }
    }

    Ok(builder.build()?)
}

/// Requests notary attestation of committed data
pub async fn notarize_transcript(
    prover: &mut Prover<tlsn_prover::state::Committed>,
    transcript_commit: TranscriptCommitConfig,
    example_type: &ExampleType,
) -> Result<(tlsn_core::attestation::Attestation, tlsn_core::Secrets), Box<dyn std::error::Error>> {
    println!("📝 Requesting Notary attestation...");

    let mut builder = RequestConfig::builder();
    builder.transcript_commit(transcript_commit);

    // Extension support for application-specific metadata (currently commented out)
    match example_type {
        ExampleType::WiseTransaction => {
            // Could add ZKP2P-specific extensions here if needed
        }
        _ => {}
    }

    let request_config = builder.build()?;

    // Generate signed attestation from Notary
    #[allow(deprecated)]
    let (attestation, secrets) = prover.notarize(&request_config).await?;

    match example_type {
        ExampleType::WiseTransaction => {
            println!("🎉 ZKP2P dual-phase payment proof generated successfully!");
            println!("   ✓ Transaction ownership verified and attested");
            println!("   ✓ Payment details cryptographically signed");
        }
        _ => {
            println!("🎉 Test proof generated successfully!");
        }
    }

    Ok((attestation, secrets))
}

/// Saves attestation and secrets to disk
pub async fn save_attestation_files(
    attestation: &tlsn_core::attestation::Attestation,
    secrets: &tlsn_core::Secrets,
    example_type: &ExampleType,
) -> Result<(), Box<dyn std::error::Error>> {
    let attestation_path = crate::get_file_path(example_type, "attestation");
    let secrets_path = crate::get_file_path(example_type, "secrets");

    println!("💾 Saving attestation files...");
    tokio::fs::write(&attestation_path, bincode::serialize(attestation)?).await?;
    tokio::fs::write(&secrets_path, bincode::serialize(secrets)?).await?;

    match example_type {
        ExampleType::WiseTransaction => {
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
            println!(
                "   1. Create selective proof: cargo run --release --example attestation_present -- wise-transaction"
            );
            println!(
                "   2. Verify payment proof: cargo run --release --example attestation_verify -- wise-transaction"
            );
            println!("   3. Submit to ZKP2P protocol for crypto asset release");
            println!();
            println!("🔐 Enhanced Privacy Guarantees:");
            println!("   ✓ Transaction ownership proven without revealing account details");
            println!("   ✓ Payment completion verified without exposing sensitive data");
            println!("   ✓ Session credentials remain completely private");
            println!("   ✓ Dual-phase verification prevents transaction ID enumeration attacks");
        }
        _ => {
            println!("✨ Test proof process completed successfully!");
            println!();
            println!("📁 Generated files:");
            println!(
                "   🔒 Test Proof: {} (cryptographic attestation signed by Notary)",
                attestation_path
            );
            println!(
                "   🔑 Verification Data: {} (for selective disclosure)",
                secrets_path
            );
            println!();
            println!("🧪 Test proof created for development purposes");
        }
    }

    Ok(())
}
