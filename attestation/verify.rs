use clap::Parser;
use std::time::Duration;
use tlsn_core::{
    CryptoProvider,
    presentation::{Presentation, PresentationOutput},
    signing::VerifyingKey,
};
use tracing::info;

use zkp2p_tlsn_rust::{
    config,
    domain::ProviderArgs,
    utils::{file_io, info},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = ProviderArgs::parse();
    let app_config =
        config::AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    info!("🔍 Verifying transaction presentation...");

    let presentation_path = file_io::get_file_path(&args.provider.to_string(), "presentation");
    let presentation: Presentation = bincode::deserialize(&std::fs::read(presentation_path)?)?;
    let VerifyingKey {
        alg,
        data: key_data,
    } = presentation.verifying_key();

    info::print_notary_info(alg, hex::encode(key_data));

    let PresentationOutput {
        server_name,
        connection_info,
        transcript,
        // extensions (Optionally, verify any custom extensions from prover/notary).
        ..
    } = presentation
        .verify(&CryptoProvider::default())
        .map_err(|e| format!("Cryptographic verification failed: {}", e))?;

    let mut partial_transcript = transcript.unwrap();
    partial_transcript.set_unauthed(app_config.unauthed_bytes.as_bytes()[0]);

    info::print_provider_info(
        &server_name.unwrap(),
        chrono::DateTime::UNIX_EPOCH + Duration::from_secs(connection_info.time),
    );

    info::print_verification_results(
        &partial_transcript.sent_unsafe(),
        &partial_transcript.received_unsafe(),
        &args.provider,
    );

    Ok(())
}
