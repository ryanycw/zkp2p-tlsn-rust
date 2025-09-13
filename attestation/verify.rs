use clap::Parser;

use zkp2p_tlsn_rust::{domain::ProviderArgs, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = ProviderArgs::parse();

    zkp2p_tlsn_rust::verify(&args.provider).await?;

    Ok(())
}
