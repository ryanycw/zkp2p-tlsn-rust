use clap::Parser;

use zkp2p_tlsn_rust::{domain::VerifyArgs, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = VerifyArgs::parse();

    zkp2p_tlsn_rust::verify(&args.provider, &args.transaction_id).await?;

    Ok(())
}
