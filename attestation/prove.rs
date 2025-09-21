use clap::Parser;

use tlsnprover::{domain, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = domain::ProveArgs::parse();

    tlsnprover::prove(
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
