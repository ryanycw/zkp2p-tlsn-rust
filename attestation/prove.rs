use clap::Parser;

use zkp2p_tlsn_rust::{domain, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = domain::ProveArgs::parse();

    zkp2p_tlsn_rust::prove(
        &args.mode,
        &args.provider,
        &args.transaction_id,
        args.profile_id.as_deref(),
        args.cookie.as_deref(),
        args.access_token.as_deref(),
    )
    .await?;

    Ok(())
}
