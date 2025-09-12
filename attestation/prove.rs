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
use tracing::{debug, info};

use zkp2p_tlsn_rust::{
    config, domain,
    utils::{file_io, info, notary, providers, text_parser},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    info::init_tracing().expect("Failed to initialize tracing");

    let args = domain::ProveArgs::parse();

    let app_config =
        config::AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    let provider_config = domain::ProviderConfig::new(
        args.provider.clone(),
        args.profile_id.clone(),
        args.transaction_id.clone(),
        args.cookie.as_ref().unwrap().clone(),
        args.access_token.as_ref().unwrap().clone(),
    );

    let server_config = app_config.server_config(args.provider.clone());

    info!(
        "Starting ZKP2P payment attestation for {} transaction {}",
        args.provider, args.transaction_id
    );

    let (attestation, secrets, (header_start, header_end), field_ranges) = if args.mode
        != domain::Mode::Present
    {
        info!(
            "Requesting notarization from {}:{}",
            app_config.notary.server.host, app_config.notary.server.port
        );

        let notary_client = NotaryClient::builder()
            .host(&app_config.notary.server.host)
            .port(app_config.notary.server.port)
            .enable_tls(app_config.notary.tls_enabled)
            .build()
            .unwrap();
        debug!("Notary client configured");

        let accepted = notary::request_notarization(
            &notary_client,
            app_config.max_sent_data,
            app_config.max_recv_data,
        )
        .await?;
        debug!("Notarization request accepted");

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
        debug!("Prover configuration built for {}", server_config.host);

        let prover = tlsn_prover::Prover::new(prover_config)
            .setup(accepted.io.compat())
            .await?;
        debug!("MPC-TLS prover initialized");

        let client_socket =
            tokio::net::TcpStream::connect((server_config.host.as_str(), server_config.port))
                .await?;
        debug!("Connected to {}:{}", server_config.host, server_config.port);

        let (mpc_tls_connection, prover_fut) = prover.connect(client_socket.compat()).await?;
        let mpc_tls_connection = TokioIo::new(mpc_tls_connection.compat());
        // Spawn the prover task to be run concurrently in the background
        let prover_task = tokio::spawn(prover_fut);
        // Attach the hyper HTTP client to the connection
        let (mut request_sender, connection) =
            hyper::client::conn::http1::handshake(mpc_tls_connection).await?;
        // Spawn the HTTP task to be run concurrently in the background
        tokio::spawn(connection);
        debug!("MPC-TLS connection established");

        providers::execute_transaction_request(
            &mut request_sender,
            &provider_config,
            &server_config,
            &app_config.user_agent,
        )
        .await?;
        debug!("Transaction request executed");

        let mut prover = prover_task.await??;
        let mut builder = TranscriptCommitConfig::builder(prover.transcript());

        let header_range = text_parser::find_host_header_range(prover.transcript().sent()).unwrap();
        builder.commit_sent(&(header_range.0..header_range.1))?;
        debug!("Committed to host header range: {:?}", header_range);

        let field_ranges =
            text_parser::find_field_ranges(prover.transcript().received(), &args.provider);
        for (start, end) in &field_ranges {
            builder.commit_recv(&(*start..*end))?;
        }
        debug!("Committed to {} payment field ranges", field_ranges.len());

        let transcript_commit = builder.build()?;
        let mut builder = RequestConfig::builder();
        builder.transcript_commit(transcript_commit);
        debug!("Attestation request built");

        let request_config = builder.build()?;
        #[allow(deprecated)]
        let (attestation, secrets) = prover.notarize(&request_config).await?;
        info!("Notarization completed successfully");

        (attestation, secrets, header_range, field_ranges)
    } else {
        info!("Loading existing attestation for presentation");
        let attestation_path = file_io::get_file_path(&args.provider.to_string(), "attestation");
        let secrets_path = file_io::get_file_path(&args.provider.to_string(), "secrets");

        let attestation: Attestation = bincode::deserialize(&std::fs::read(attestation_path)?)?;
        let secrets: Secrets = bincode::deserialize(&std::fs::read(secrets_path)?)?;
        debug!("Loaded attestation and secrets from disk");

        let header_range =
            text_parser::find_host_header_range(secrets.transcript().sent()).unwrap();
        let field_ranges =
            text_parser::find_field_ranges(secrets.transcript().received(), &args.provider);
        debug!(
            "Parsed {} field ranges for selective disclosure",
            field_ranges.len()
        );

        (attestation, secrets, header_range, field_ranges)
    };

    if args.mode == domain::Mode::Prove {
        file_io::save_file(&provider_config.provider_type, "attestation", &attestation).await?;
        file_io::save_file(&provider_config.provider_type, "secrets", &secrets).await?;
        info!("Attestation completed and saved");
        return Ok(());
    }

    info!("Building selective disclosure presentation");
    let mut builder = secrets.transcript_proof_builder();
    builder.reveal_sent(&(header_start..header_end))?;
    for (start, end) in &field_ranges {
        builder.reveal_recv(&(*start..*end))?;
    }
    debug!(
        "Configured revelations: header + {} field ranges",
        field_ranges.len()
    );

    let transcript_proof = builder.build()?;
    let crypto_provider = CryptoProvider::default();
    let mut builder = attestation.presentation_builder(&crypto_provider);
    builder
        .identity_proof(secrets.identity_proof())
        .transcript_proof(transcript_proof);
    let presentation: Presentation = builder.build()?;
    debug!("Presentation built successfully");

    file_io::save_file(&args.provider, "presentation", &presentation).await?;
    debug!("Presentation saved to disk");

    info!("Presentation completed and saved");
    info!("Next: Run verification with 'cargo run --release --bin zkp2p-verify'");

    return Ok(());
}
