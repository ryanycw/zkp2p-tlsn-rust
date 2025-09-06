use clap::Parser;

use tlsn_core::{CryptoProvider, Secrets, attestation::Attestation, presentation::Presentation};
use zkp2p_tlsn_rust::{domain::ProviderArgs, utils::http::find_field_ranges};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = ProviderArgs::parse();

    create_presentation(&args.provider).await
}

async fn create_presentation(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let attestation_path = zkp2p_tlsn_rust::get_file_path(provider, "attestation");
    let secrets_path = zkp2p_tlsn_rust::get_file_path(provider, "secrets");

    println!("🎭 Creating selective disclosure presentation for Wise transaction...");
    println!("   Reading attestation from: {}", attestation_path);
    println!("   Reading secrets from: {}", secrets_path);

    // Read attestation & secretsfrom disk.
    let attestation: Attestation = bincode::deserialize(&std::fs::read(attestation_path)?)?;
    let secrets: Secrets = bincode::deserialize(&std::fs::read(secrets_path)?)?;

    println!("🔧 Creating selective disclosure presentation for chunked response data...");
    let mut builder = secrets.transcript_proof_builder();

    // Parse HTTP response to find payment field ranges
    println!("📊 Processing response for ZKP2P verification...");
    let sent_data = secrets.transcript().sent();
    let recv_data = secrets.transcript().received();
    builder.reveal_sent(&(0..sent_data.len()))?;

    let field_ranges = find_field_ranges(recv_data);

    println!("🔍 Found {} payment fields to reveal:", field_ranges.len());
    for (start, end, field_name) in &field_ranges {
        println!("     Revealing {}: range {}..{}", field_name, start, end);
        builder.reveal_recv(&(*start..*end))?;
        println!("     ✅ Revealed: {}", field_name);
    }

    println!("   ✅ ZKP2P essential fields revealed, sensitive data remains private");

    let transcript_proof = builder.build()?;

    // Use default crypto provider to build the presentation
    let crypto_provider = CryptoProvider::default();
    let mut builder = attestation.presentation_builder(&crypto_provider);

    builder
        .identity_proof(secrets.identity_proof())
        .transcript_proof(transcript_proof);

    let presentation: Presentation = builder.build()?;

    let presentation_path = zkp2p_tlsn_rust::get_file_path(provider, "presentation");

    // Write the presentation to disk
    std::fs::write(&presentation_path, bincode::serialize(&presentation)?)?;

    println!("🎉 ZKP2P chunked presentation with selective disclosure created successfully!");
    println!("📁 Presentation written to: {}", presentation_path);
    println!();
    println!("🔒 Privacy Summary:");
    println!("   • Session cookies and tokens: HIDDEN");
    println!("   • Account numbers and personal details: HIDDEN");
    println!("   • Payment amount, currency, status: REVEALED for verification");
    println!("   • Transaction ID and date: REVEALED for matching");
    println!();
    println!("🎯 Next step: Run verification with:");
    println!("   cargo run --release --bin zkp2p-verify");

    Ok(())
}
