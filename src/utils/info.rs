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
    info!("Cryptographic verification details:");
    info!("Algorithm: {}, Key: {}", alg, key_data);
    warn!("Only proceed if you trust the Notary that generated this key");
}

pub fn print_provider_info(
    server_name: impl std::fmt::Display,
    session_time: impl std::fmt::Display,
) {
    info!("Verified connection: {} at {}", server_name, session_time);
}

pub fn print_verification_results(request_data: &[u8], response_data: &[u8], provider: &Provider) {
    let request = String::from_utf8_lossy(request_data);
    let response = String::from_utf8_lossy(response_data);

    let field_ranges = find_field_ranges(&response_data, &provider);

    if field_ranges.len() > 0 {
        info!(
            "Payment verification successful: {} fields verified",
            field_ranges.len()
        );
    } else {
        warn!("No ZKP2P fields found in revealed data");
        info!("Request: {}", request);
        info!("Response: {}", response);
    }

    info!("ZKP2P verification completed - payment proof validated");
    info!("Proof ready for smart contract submission");
}
