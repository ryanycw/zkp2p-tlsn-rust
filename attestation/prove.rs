use std::env;
use clap::Parser;
use hyper_util::rt::TokioIo;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tlsn_formats::http::HttpTranscript;

use zkp2p_tlsn_rust::{
    ExampleType,
    attestation, http, notary, providers, tls,
};

// ZKP2P TLSNotary Prover Implementation for Wise Payment Verification
//
// This implementation demonstrates the TLSNotary three-party protocol (Prover, Notary, Verifier)
// developed by Ethereum Foundation's Privacy and Scaling Explorations (PSE) team.
//
// ZKP2P Integration Purpose:
// - Cryptographically prove completion of fiat payment through Wise
// - Enable trustless verification of payment without exposing sensitive data
// - Support decentralized on/off-ramp protocol for crypto-fiat exchanges
//
// TLSNotary Protocol Flow:
// Phase 1: MPC-TLS Connection - Prover collaborates with Notary via MPC to establish
//          secret-shared TLS session keys with wise.com
// Phase 2: Payment Verification - Authenticated web requests occur with cryptographic guarantees
// Phase 3: Notarization - Notary creates cryptographic payment proofs without seeing plaintext
//
// Critical: Wise.com sees only standard browser traffic - Notary is transparent

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Type of payment proof to generate.
    /// - Json/Html/Authenticated: Test fixtures for local development
    /// - WiseTransaction: ZKP2P Wise payment verification via zkTLS/MPC-TLS
    #[clap(default_value_t, value_enum)]
    example_type: ExampleType,
    
    /// Wise profile ID (required for wise-transaction)
    #[clap(long)]
    wise_profile_id: Option<String>,
    
    /// Wise transaction ID to prove (required for wise-transaction)
    #[clap(long)]
    wise_transaction_id: Option<String>,
    
    /// Wise session cookie (required for wise-transaction)
    #[clap(long)]
    wise_cookie: Option<String>,
    
    /// Wise access token (required for wise-transaction)
    #[clap(long)]
    wise_access_token: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables from .env file
    dotenv::dotenv().ok();
    
    let args = Args::parse();
    
    // Prepare request configuration based on example type
    let (initial_uri, extra_headers, server_config) = match &args.example_type {
        ExampleType::Json => ("/formats/json", vec![], providers::ServerConfig::test_fixture()),
        ExampleType::Html => ("/formats/html", vec![], providers::ServerConfig::test_fixture()),
        ExampleType::Authenticated => (
            "/protected", 
            vec![("Authorization", "random_auth_token")], 
            providers::ServerConfig::test_fixture()
        ),
        ExampleType::WiseTransaction => {
            // Validate required arguments
            let wise_config = providers::wise::WiseConfig::new(
                args.wise_profile_id.clone()
                    .expect("--wise-profile-id required for wise-transaction"),
                args.wise_transaction_id.clone()
                    .expect("--wise-transaction-id required for wise-transaction"),
                args.wise_cookie.clone()
                    .expect("--wise-cookie required for wise-transaction"),
                args.wise_access_token.clone()
                    .expect("--wise-access-token required for wise-transaction"),
            );
            
            // Return transaction list endpoint for Phase 1 verification
            (providers::wise::WiseConfig::transaction_list_endpoint(), 
             wise_config.auth_headers(), 
             wise_config.server_config)
        }
    };
    
    notarize(initial_uri, extra_headers, &args.example_type, &args, server_config).await
}

/// ZKP2P Payment Verification via TLSNotary MPC-TLS - Dual Phase Implementation
/// 
/// Implements enhanced TLSNotary protocol for secure payment proof generation:
/// Phase 1: Transaction Ownership Verification - Proves transaction exists in user's list
/// Phase 2: Transaction Details Attestation - Attests specific payment data
/// Phase 3: MPC-TLS Connection Setup - Secret-share TLS keys with Notary  
/// Phase 4: Dual-Request Execution - Two authenticated requests over single MPC-TLS session
/// Phase 5: Cryptographic Proof Generation - Notary signs combined attestation
async fn notarize(
    initial_uri: &str,
    extra_headers: Vec<(&str, &str)>,
    example_type: &ExampleType,
    args: &Args,
    server_config: providers::ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🚀 Starting ZKP2P payment verification via TLSNotary...");
    
    // Configure notary connection
    let notary_config = notary::NotaryConfig::from_env(example_type);
    let notary_client = notary_config.build_client();
    
    // Request notarization from Notary server
    let accepted = notary::request_notarization(
        &notary_client,
        zkp2p_tlsn_rust::MAX_SENT_DATA,
        zkp2p_tlsn_rust::MAX_RECV_DATA,
    ).await?;
    
    // Configure TLS certificate verification for target server
    let crypto_provider = tls::create_crypto_provider(example_type);
    
    // Build prover configuration for MPC-TLS
    let prover_config = tls::build_prover_config(
        server_config.server_name,
        zkp2p_tlsn_rust::MAX_SENT_DATA,
        zkp2p_tlsn_rust::MAX_RECV_DATA,
        crypto_provider,
        example_type,
    )?;
    
    // Initialize MPC-TLS Prover with Notary collaboration
    let prover = tls::setup_mpc_tls_prover(
        prover_config,
        accepted.io.compat()
    ).await?;
    
    // Establish TCP connection to target server
    let client_socket = tls::connect_to_server(&server_config.host, server_config.port).await?;
    
    // Establish MPC-TLS connection with target server
    // CRITICAL: This creates a three-way MPC connection (Prover ↔ Notary ↔ Server)
    // - Prover and Notary secret-share TLS session keys via MPC
    // - Target server sees standard TLS 1.2 connection (Notary is transparent)
    // - All data encryption/decryption occurs through MPC with Notary
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

    // Execute requests based on example type
    match example_type {
        ExampleType::WiseTransaction => {
            // Get Wise configuration for dual-phase requests
            let wise_config = providers::wise::WiseConfig::new(
                args.wise_profile_id.as_ref().unwrap().clone(),
                args.wise_transaction_id.as_ref().unwrap().clone(),
                args.wise_cookie.as_ref().unwrap().clone(),
                args.wise_access_token.as_ref().unwrap().clone(),
            );
            
            // Execute dual-phase Wise transaction verification
            providers::wise::execute_dual_phase_requests(
                &mut request_sender,
                &wise_config,
                server_config.server_name,
            ).await?;
        }
        _ => {
            // Single-phase request for test fixtures
            let request = http::build_request(
                initial_uri,
                server_config.server_name, 
                &extra_headers,
                "Test fixture request"
            )?;
            
            let response = request_sender.send_request(request).await?;
            
            if response.status() != hyper::StatusCode::OK {
                return Err(format!("❌ Server returned error status: {}", response.status()).into());
            }
            
            println!("✅ Test response received successfully");
        }
    }

    // Complete MPC-TLS session and retrieve transcript
    println!("🏁 Completing MPC-TLS session...");
    let mut prover = prover_task.await??;

    // Parse the HTTP transcript captured through MPC-TLS
    let transcript = HttpTranscript::parse(prover.transcript())?;
    println!("📋 HTTP transcript parsed successfully");
    
    // Analyze transcript based on example type
    attestation::analyze_transcript(&transcript, example_type)?;
    
    // Create cryptographic commitments to transcript data
    let transcript_commit = attestation::create_transcript_commitment(
        prover.transcript(),
        &transcript,
        example_type,
    )?;
    
    // Request Notary attestation of committed data
    // CRITICAL: Notary creates cryptographic signatures without seeing:
    // - Plaintext payment details or transaction list data
    // - Session credentials (Cookie, X-Access-Token)  
    // - Server identity (wise.com)
    // - Personal information (account numbers, names, etc.)
    // - The Notary can attest to both transaction ownership AND payment details
    let (attestation_data, secrets) = attestation::notarize_transcript(
        &mut prover,
        transcript_commit,
        example_type,
    ).await?;
    
    // Save attestation and secrets to disk
    attestation::save_attestation_files(
        &attestation_data,
        &secrets,
        example_type,
    ).await?;
    
    Ok(())
}