use clap::Parser;

use tlsnprover::{config::AppConfig, domain::VerifyArgs, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = VerifyArgs::parse();
    let app_config =
        AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    tlsnprover::verify(&args.url, &app_config.unauthed_bytes).await?;

    Ok(())
}
