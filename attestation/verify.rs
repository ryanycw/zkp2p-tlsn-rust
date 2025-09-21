use clap::Parser;

use tlsnprover::{domain::VerifyArgs, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = VerifyArgs::parse();

    tlsnprover::verify(&args.provider, &args.transaction_id).await?;

    Ok(())
}
