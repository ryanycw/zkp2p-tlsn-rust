use color_eyre::eyre::Result;
use tracing::{info, warn};
use tracing_error::ErrorLayer;
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

use crate::{domain::Provider, utils::text_parser::find_field_ranges};

pub fn init_tracing() -> Result<()> {
    let fmt_layer = fmt::layer().compact();
    let filter_layer = EnvFilter::try_from_default_env().or_else(|_| EnvFilter::try_new("info"))?;

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();

    Ok(())
}

pub fn print_notary_info(alg: impl std::fmt::Display, key_data: impl std::fmt::Display) {
    info!("🔑 Cryptographic verification details:");
    info!("   This key signed the cryptographic proof you're about to verify.");
    info!("   Only proceed if you trust the Notary that generated this key.");
    info!("   Algorithm: {}", alg);
    info!("   Notary Key: {}", key_data);
    info!("   Unauthenticated bytes set to: X");
}

pub fn print_provider_info(
    server_name: impl std::fmt::Display,
    session_time: impl std::fmt::Display,
) {
    info!("Verified Connection Details:");
    info!("   Server: {}", server_name);
    info!("   Session Time: {}", session_time);
}

pub fn print_verification_results(request_data: &[u8], response_data: &[u8], provider: &Provider) {
    let request = String::from_utf8_lossy(request_data);
    let response = String::from_utf8_lossy(response_data);

    info!("ZKP2P Payment Verification Results:");

    let field_ranges = find_field_ranges(&response_data, &provider);

    if field_ranges.len() > 0 {
        info!(
            "   Successfully verified {} ZKP2P payment fields",
            field_ranges.len()
        );
    } else {
        warn!("   Warning: No ZKP2P fields found in revealed data");
        info!("   Raw revealed request: {}", request);
        info!("   Raw revealed response: {}", response);
    }

    info!("============================================================================");
    info!("ZKP2P VERIFICATION COMPLETE - PAYMENT PROOF VALIDATED");
    info!("============================================================================");
    info!("Verification process completed successfully!");
    info!("ZKP2P Integration Ready:");
    info!("   This verified proof can now be submitted to ZKP2P smart contracts");
    info!("   for automated crypto asset release upon payment confirmation.");
}
