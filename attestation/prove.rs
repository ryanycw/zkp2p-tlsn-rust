use clap::Parser;

use tlsnprover::{config::AppConfig, domain, utils::info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = domain::ProveArgs::parse();
    let app_config =
        AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    tlsnprover::prove(
        &args.mode,
        &args.url,
        args.cookie.as_deref(),
        args.access_token.as_deref(),
        &app_config.user_agent,
        &app_config.wise.host,
        app_config.wise.port,
        &app_config.notary.server.host,
        app_config.notary.server.port,
        app_config.notary.tls_enabled,
        app_config.max_sent_data,
        app_config.max_recv_data,
    )
    .await?;

    Ok(())
}
