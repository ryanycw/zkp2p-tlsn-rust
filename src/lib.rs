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

use std::ffi::{CStr, CString};
use std::os::raw::c_char;

pub mod config;
pub mod domain;
pub mod utils;

use config::AppConfig;
use domain::{Provider, ProviderConfig};
use utils::{file_io, notary, providers, text_parser};

use crate::domain::Mode;

pub async fn prove(
    mode: &Mode,
    provider: &Provider,
    profile_id: Option<&str>,
    transaction_id: &str,
    cookie: Option<&str>,
    access_token: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let app_config =
        AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    let provider_config = ProviderConfig::new(
        provider.clone(),
        profile_id.map(|s| s.to_string()),
        transaction_id.to_string(),
        cookie.unwrap_or("").to_string(),
        access_token.unwrap_or("").to_string(),
    );

    let server_config = app_config.server_config(provider.clone());

    info!(
        "Starting ZKP2P payment attestation for {} transaction {}",
        provider, transaction_id
    );

    let (attestation, secrets, (header_start, header_end), field_ranges) = if *mode != Mode::Present
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
        let prover_task = tokio::spawn(prover_fut);
        let (mut request_sender, connection) =
            hyper::client::conn::http1::handshake(mpc_tls_connection).await?;
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
            text_parser::find_field_ranges(prover.transcript().received(), &provider);
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
        let attestation_path = file_io::get_file_path(&provider.to_string(), "attestation");
        let secrets_path = file_io::get_file_path(&provider.to_string(), "secrets");

        let attestation: Attestation = bincode::deserialize(&std::fs::read(attestation_path)?)?;
        let secrets: Secrets = bincode::deserialize(&std::fs::read(secrets_path)?)?;
        debug!("Loaded attestation and secrets from disk");

        let header_range =
            text_parser::find_host_header_range(secrets.transcript().sent()).unwrap();
        let field_ranges =
            text_parser::find_field_ranges(secrets.transcript().received(), &provider);
        debug!(
            "Parsed {} field ranges for selective disclosure",
            field_ranges.len()
        );

        (attestation, secrets, header_range, field_ranges)
    };

    if *mode == Mode::Prove {
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

    file_io::save_file(&provider, "presentation", &presentation).await?;
    debug!("Presentation saved to disk");

    info!("Presentation completed and saved");
    info!("Next: Run verification with 'cargo run --release --bin zkp2p-verify'");

    Ok(())
}

pub async fn verify(provider: &Provider) -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;
    use tlsn_core::{
        presentation::{Presentation, PresentationOutput},
        signing::VerifyingKey,
    };

    let app_config =
        AppConfig::new().map_err(|e| format!("Failed to load configuration: {}", e))?;

    info!("🔍 Verifying transaction presentation...");

    let presentation_path = file_io::get_file_path(&provider.to_string(), "presentation");
    let presentation: Presentation = bincode::deserialize(&std::fs::read(presentation_path)?)?;
    let VerifyingKey {
        alg,
        data: key_data,
    } = presentation.verifying_key();

    utils::info::print_notary_info(alg, hex::encode(key_data));

    let PresentationOutput {
        server_name,
        connection_info,
        transcript,
        ..
    } = presentation
        .verify(&CryptoProvider::default())
        .map_err(|e| format!("Cryptographic verification failed: {}", e))?;

    let mut partial_transcript = transcript.unwrap();
    partial_transcript.set_unauthed(app_config.unauthed_bytes.as_bytes()[0]);

    utils::info::print_provider_info(
        &server_name.unwrap(),
        chrono::DateTime::UNIX_EPOCH + Duration::from_secs(connection_info.time),
    );

    utils::info::print_verification_results(
        &partial_transcript.sent_unsafe(),
        &partial_transcript.received_unsafe(),
        &provider,
    );

    Ok(())
}

fn mode_from_int(mode: i32) -> Option<Mode> {
    match mode {
        0 => Some(Mode::Prove),
        1 => Some(Mode::Present),
        2 => Some(Mode::ProveToPresent),
        _ => None,
    }
}

fn provider_from_int(provider: i32) -> Option<Provider> {
    match provider {
        0 => Some(Provider::Wise),
        1 => Some(Provider::PayPal),
        _ => None,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_prove(
    mode: i32,
    provider: i32,
    profile_id: *const c_char,
    transaction_id: *const c_char,
    cookie: *const c_char,
    access_token: *const c_char,
) -> i32 {
    let mode_enum = match mode_from_int(mode) {
        Some(m) => m,
        None => return 1,
    };

    let provider_enum = match provider_from_int(provider) {
        Some(p) => p,
        None => return 1,
    };

    if transaction_id.is_null() {
        return 1;
    }

    let transaction_id_str = unsafe {
        match CStr::from_ptr(transaction_id).to_str() {
            Ok(s) => s,
            Err(_) => return 1,
        }
    };

    let profile_id_str = if profile_id.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(profile_id).to_str() {
                Ok(s) => Some(s),
                Err(_) => return 1,
            }
        }
    };

    let cookie_str = if cookie.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(cookie).to_str() {
                Ok(s) => Some(s),
                Err(_) => return 1,
            }
        }
    };

    let access_token_str = if access_token.is_null() {
        None
    } else {
        unsafe {
            match CStr::from_ptr(access_token).to_str() {
                Ok(s) => Some(s),
                Err(_) => return 1,
            }
        }
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => return 1,
    };

    match rt.block_on(prove(
        &mode_enum,
        &provider_enum,
        profile_id_str,
        transaction_id_str,
        cookie_str,
        access_token_str,
    )) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_verify(provider: i32) -> i32 {
    let provider_enum = match provider_from_int(provider) {
        Some(p) => p,
        None => return 1,
    };

    let rt = match tokio::runtime::Runtime::new() {
        Ok(runtime) => runtime,
        Err(_) => return 1,
    };

    match rt.block_on(verify(&provider_enum)) {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ffi_free_string(ptr: *mut c_char) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        let _ = CString::from_raw(ptr);
    }
}
