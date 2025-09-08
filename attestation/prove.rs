use clap::Parser;
use hyper_util::rt::TokioIo;
use notary_client::NotaryClient;
use tlsn_common::config::ProtocolConfig;
use tlsn_core::{
    CryptoProvider, Secrets, attestation::Attestation, presentation::Presentation,
    request::RequestConfig, transcript::TranscriptCommitConfig,
};
use tlsn_prover::ProverConfig;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};

use zkp2p_tlsn_rust::{
    config::AppConfig,
    domain,
    utils::{http, notary, providers, transcript},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = domain::ProveArgs::parse();

    match args.mode {
        domain::Mode::Prove => {
            prove(&args).await?;
        }
        domain::Mode::Present => {
            create_presentation(&args.provider).await?;
        }
        domain::Mode::ProveToPresent => {
            prove(&args).await?;
            create_presentation(&args.provider).await?;
        }
    }

    Ok(())
}

async fn prove(args: &domain::ProveArgs) -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();

    let app_config =
        AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    let provider_type = match args.provider.as_str() {
        "wise" => domain::ProviderType::Wise,
        "paypal" => domain::ProviderType::PayPal,
        provider => return Err(format!("Unsupported platform: {}", provider).into()),
    };

    let provider_config = domain::ProviderConfig::new(
        provider_type.clone(),
        args.profile_id.clone(),
        args.transaction_id.clone(),
        args.cookie.as_ref().unwrap().clone(),
        args.access_token.as_ref().unwrap().clone(),
    );

    let server_config = app_config.server_config(provider_type);

    notarize(&provider_config, &server_config, &app_config).await
}

async fn notarize(
    provider_config: &domain::ProviderConfig,
    server_config: &domain::server::ServerConfig,
    app_config: &AppConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    println!("🚀 Starting ZKP2P payment verification via TLSNotary...");

    // Configure notary connection
    let notary_client = NotaryClient::builder()
        .host(&app_config.notary.server.host)
        .port(app_config.notary.server.port)
        .enable_tls(app_config.notary.tls_enabled)
        .build()
        .unwrap();

    // Request notarization from Notary server
    let accepted = notary::request_notarization(
        &notary_client,
        app_config.max_sent_data,
        app_config.max_recv_data,
    )
    .await?;

    // Build prover configuration for MPC-TLS
    let prover_config = ProverConfig::builder()
        .server_name(server_config.host.as_str())
        .protocol_config(
            ProtocolConfig::builder()
                .max_sent_data(app_config.max_sent_data)
                .max_recv_data(app_config.max_recv_data)
                .build()?,
        )
        .crypto_provider(tlsn_core::CryptoProvider::default())
        .build()
        .ok()
        .ok_or("Failed to build prover config")?;

    // Initialize MPC-TLS Prover with Notary collaboration
    println!("🤝 Setting up MPC-TLS collaboration with Notary...");
    let prover = tlsn_prover::Prover::new(prover_config)
        .setup(accepted.io.compat())
        .await?;

    // Establish TCP connection to target server
    let client_socket =
        tokio::net::TcpStream::connect((server_config.host.as_str(), server_config.port)).await?;
    println!("🔐 Establishing MPC-TLS connection (Prover ↔ Notary ↔ Server)...");
    let (mpc_tls_connection, prover_fut) = prover.connect(client_socket.compat()).await?;
    let mpc_tls_connection = TokioIo::new(mpc_tls_connection.compat());
    println!("✅ MPC-TLS connection established - Notary transparent to server");

    // Spawn the prover task to be run concurrently in the background
    let prover_task = tokio::spawn(prover_fut);

    // Attach the hyper HTTP client to the connection
    let (mut request_sender, connection) =
        hyper::client::conn::http1::handshake(mpc_tls_connection).await?;

    // Spawn the HTTP task to be run concurrently in the background
    tokio::spawn(connection);

    println!("🔄 Executing transaction request...");
    // Execute transaction request using unified function
    providers::execute_transaction_request(
        &mut request_sender,
        &provider_config,
        &server_config,
        &app_config.user_agent,
    )
    .await?;

    println!("🏁 Transaction request executed - Completing MPC-TLS session...");
    let mut prover = prover_task.await??;
    let transcript = prover.transcript();
    println!("🔄 Committing to transcript...");
    let mut builder = TranscriptCommitConfig::builder(transcript);

    // Commit to only the host header in the sent data (the request)
    let sent_data = prover.transcript().sent();
    let (start, end) = http::find_host_header_range(sent_data).unwrap();
    println!("🔍 Found host header at range {}..{}", start, end);
    builder.commit_sent(&(start..end))?;

    // Parse response data to find specific payment field ranges
    let recv_data = prover.transcript().received();
    let field_ranges = http::find_field_ranges(recv_data);
    println!("🔍 Found {} payment fields to commit:", field_ranges.len());
    for (start, end, field_name) in &field_ranges {
        println!("   - {}: range {}..{}", field_name, start, end);
        builder.commit_recv(&(*start..*end))?;
    }
    let transcript_commit = builder.build()?;

    // Build an attestation request.
    println!("🔄 Building attestation request...");
    let mut builder = RequestConfig::builder();
    builder.transcript_commit(transcript_commit);
    let request_config = builder.build()?;

    #[allow(deprecated)]
    let (attestation, secrets) = prover.notarize(&request_config).await?;

    println!("Notarization complete!");
    transcript::save_attestation_files(&provider_config.provider_type, &attestation, &secrets)
        .await?;

    Ok(())
}

async fn create_presentation(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
    let attestation_path = zkp2p_tlsn_rust::get_file_path(provider, "attestation");
    let secrets_path = zkp2p_tlsn_rust::get_file_path(provider, "secrets");

    println!("🎭 Creating selective disclosure presentation for Wise transaction...");
    println!("   Reading attestation from: {}", attestation_path);
    println!("   Reading secrets from: {}", secrets_path);

    // Read attestation & secrets from disk.
    let attestation: Attestation = bincode::deserialize(&std::fs::read(attestation_path)?)?;
    let secrets: Secrets = bincode::deserialize(&std::fs::read(secrets_path)?)?;

    println!("🔧 Creating selective disclosure presentation for chunked response data...");
    let mut builder = secrets.transcript_proof_builder();

    // Parse HTTP request and response for selective disclosure
    println!("📊 Processing request and response for ZKP2P verification...");
    let sent_data = secrets.transcript().sent();
    let recv_data = secrets.transcript().received();

    let (start, end) = http::find_host_header_range(sent_data).unwrap();
    println!("🔍 Found host header at range {}..{}", start, end);
    builder.reveal_sent(&(start..end))?;

    let field_ranges = http::find_field_ranges(recv_data);
    println!("🔍 Found {} payment fields to reveal:", field_ranges.len());
    for (start, end, field_name) in &field_ranges {
        println!("     Revealing {}: range {}..{}", field_name, start, end);
        builder.reveal_recv(&(*start..*end))?;
        println!("     ✅ Revealed: {}", field_name);
    }

    println!("   ✅ ZKP2P essential fields revealed, sensitive data remains private");

    let transcript_proof = builder.build()?;

    // Use default crypto provider to build the presentation
    let crypto_provider = CryptoProvider::default();
    let mut builder = attestation.presentation_builder(&crypto_provider);

    builder
        .identity_proof(secrets.identity_proof())
        .transcript_proof(transcript_proof);

    let presentation: Presentation = builder.build()?;

    let presentation_path = zkp2p_tlsn_rust::get_file_path(provider, "presentation");

    // Write the presentation to disk
    std::fs::write(&presentation_path, bincode::serialize(&presentation)?)?;

    println!("🎉 ZKP2P chunked presentation with selective disclosure created successfully!");
    println!("📁 Presentation written to: {}", presentation_path);
    println!();
    println!("🔒 Privacy Summary:");
    println!("   • Session cookies and tokens: HIDDEN");
    println!("   • Account numbers and personal details: HIDDEN");
    println!("   • Request headers except host: HIDDEN");
    println!("   • Host header: REVEALED (host: wise.com)");
    println!("   • Payment amount, currency, status: REVEALED for verification");
    println!("   • Transaction ID and date: REVEALED for matching");
    println!();
    println!("🎯 Next step: Run verification with:");
    println!("   cargo run --release --bin zkp2p-verify");

    Ok(())
}
