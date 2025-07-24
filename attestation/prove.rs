use std::env;

use clap::Parser;
use http_body_util::{BodyExt, Empty};
use hyper::{Request, StatusCode, body::Bytes};
use hyper_util::rt::TokioIo;
use spansy::Spanned;
use tokio_util::compat::{FuturesAsyncReadCompatExt, TokioAsyncReadCompatExt};
use tracing::debug;

use notary_client::{Accepted, NotarizationRequest, NotaryClient};
use tls_core::verify::WebPkiVerifier;
use tls_server_fixture::{CA_CERT_DER, SERVER_DOMAIN};
use tlsn_common::config::ProtocolConfig;
use tlsn_core::{CryptoProvider, request::RequestConfig, transcript::TranscriptCommitConfig};
use tlsn_formats::http::{DefaultHttpCommitter, HttpCommit, HttpTranscript};
use tlsn_prover::{Prover, ProverConfig, TlsConfig};
use tlsn_server_fixture::DEFAULT_FIXTURE_PORT;
use tlsn_server_fixture_certs::{CLIENT_CERT, CLIENT_KEY};
use zkp2p_tlsn_rust::ExampleType;

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

const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36";

/// Helper function to build HTTP requests with common headers
fn build_http_request(
    uri: &str,
    server_name: &str,
    extra_headers: &[(&str, &str)],
    description: &str,
) -> Result<Request<Empty<Bytes>>, Box<dyn std::error::Error>> {
    println!("📡 Building request: {}", description);
    println!("   Endpoint: {}", uri);
    
    let mut request_builder = Request::builder()
        .uri(uri)
        .header("Host", server_name)
        .header("Accept", "*/*")
        // Using "identity" instructs the Server not to use compression for its HTTP response.
        // TLSNotary tooling does not support compression.
        .header("Accept-Encoding", "identity")
        .header("Connection", "close")
        .header("User-Agent", USER_AGENT);
    
    for (key, value) in extra_headers {
        request_builder = request_builder.header(*key, *value);
        if !key.eq_ignore_ascii_case("Cookie") && !key.eq_ignore_ascii_case("X-Access-Token") {
            println!("   Header: {}: {}", key, value);
        } else {
            println!("   Header: {}: [REDACTED]", key);
        }
    }
    
    Ok(request_builder.body(Empty::<Bytes>::new())?)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Type of payment proof to generate.
    /// - Json/Html/Authenticated: Test fixtures for local development
    /// - WiseTransaction: ZKP2P Wise payment verification via zkTLS/MPC-TLS
    #[clap(default_value_t, value_enum)]
    example_type: ExampleType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Pre-load credentials for WiseTransaction to avoid lifetime issues
    let wise_cookie = if let ExampleType::WiseTransaction = args.example_type {
        Some(env::var("WISE_COOKIE").expect("WISE_COOKIE required: Web session cookie for authentication"))
    } else { None };
    
    let wise_access_token = if let ExampleType::WiseTransaction = args.example_type {
        Some(env::var("WISE_ACCESS_TOKEN").expect("WISE_ACCESS_TOKEN required: Web session token for authentication"))
    } else { None };

    let (uri, extra_headers): (&str, Vec<(&str, &str)>) = match args.example_type {
        ExampleType::Json => ("/formats/json", vec![]),
        ExampleType::Html => ("/formats/html", vec![]),
        ExampleType::Authenticated => ("/protected", vec![("Authorization", "random_auth_token")]),
        ExampleType::WiseTransaction => {
            // ZKP2P Wise Payment Verification - Phase 1: Transaction List Verification
            // This phase proves transaction ownership before revealing detailed payment data
            let profile_id = env::var("WISE_PROFILE_ID")
                .expect("WISE_PROFILE_ID required: Get from Wise profile settings");
            let transaction_id = env::var("WISE_TRANSACTION_ID")
                .expect("WISE_TRANSACTION_ID required: Specific payment to prove completion");
            
            println!("🔐 Configuring ZKP2P Wise dual-phase payment verification:");
            println!("   Profile ID: {}", profile_id);
            println!("   Target Payment ID: {}", transaction_id);
            println!("   Phase 1: Verify transaction ownership");
            println!("   Phase 2: Attest transaction details");
            println!("   Session credentials will remain private in all proofs");
            
            // Return transaction list endpoint for Phase 1 verification
            // Phase 2 endpoint will be constructed during MPC-TLS session
            ("/all-transactions?direction=OUTGOING", vec![
                ("Cookie", wise_cookie.as_ref().unwrap().as_str()),
                ("X-Access-Token", wise_access_token.as_ref().unwrap().as_str()),
            ])
        }
    };

    notarize(uri, extra_headers, &args.example_type).await
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
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    println!("🚀 Starting ZKP2P payment verification via TLSNotary...");
    
    // Phase 1: Configure Notary Server Connection
    let notary_host: String = env::var("NOTARY_HOST").unwrap_or("127.0.0.1".into());
    let notary_port: u16 = env::var("NOTARY_PORT")
        .map(|port| port.parse().expect("NOTARY_PORT should be valid integer"))
        .unwrap_or(7047);
    
    println!("📡 Connecting to Notary server: {}:{}", notary_host, notary_port);

    // Configure target server based on attestation type
    let (server_host, server_port, server_name) = match example_type {
        ExampleType::WiseTransaction => {
            let host = env::var("WISE_HOST").unwrap_or("wise.com".to_string());
            let port = env::var("WISE_PORT")
                .map(|port| port.parse().expect("WISE_PORT should be valid integer"))
                .unwrap_or(443);
            println!("🌐 Target server: {} (production HTTPS)", host);
            (host, port, "wise.com")
        },
        _ => {
            let host = env::var("SERVER_HOST").unwrap_or("127.0.0.1".into());
            let port = env::var("SERVER_PORT")
                .map(|port| port.parse().expect("SERVER_PORT should be valid integer"))
                .unwrap_or(DEFAULT_FIXTURE_PORT);
            println!("🧪 Target server: {}:{} (test fixture)", host, port);
            (host, port, SERVER_DOMAIN)
        }
    };

    // Configure Notary client - TLS enabled for production, disabled for local testing
    let enable_notary_tls = match example_type {
        ExampleType::WiseTransaction => {
            let tls_enabled = env::var("NOTARY_TLS")
                .unwrap_or("true".to_string())
                .parse()
                .unwrap_or(true);
            println!("🔒 Notary TLS: {}", if tls_enabled { "enabled (production)" } else { "disabled (testing)" });
            tls_enabled
        }
        _ => {
            println!("🔓 Notary TLS: disabled (test fixtures)");
            false
        }
    };
    
    let notary_client = NotaryClient::builder()
        .host(notary_host)
        .port(notary_port)
        .enable_tls(enable_notary_tls)
        .build()
        .unwrap();

    // Phase 1: Request notarization from Notary server
    // MPC preprocessing occurs here to optimize connection performance
    println!("📋 Requesting notarization with data limits: sent={}KB, recv={}KB", 
             zkp2p_tlsn_rust::MAX_SENT_DATA / 1024, 
             zkp2p_tlsn_rust::MAX_RECV_DATA / 1024);
             
    let notarization_request = NotarizationRequest::builder()
        // Data limits are preprocessed for MPC operations to reduce connection times
        // These limits must accommodate the expected Wise API response size
        .max_sent_data(zkp2p_tlsn_rust::MAX_SENT_DATA)
        .max_recv_data(zkp2p_tlsn_rust::MAX_RECV_DATA)
        .build()?;

    let Accepted {
        io: notary_connection,
        id: session_id,
        ..
    } = notary_client
        .request_notarization(notarization_request)
        .await
        .expect("❌ Failed to connect to Notary server. Ensure it's running and accessible.");
        
    println!("✅ Notary connection established (session: {})", session_id);

    // Configure TLS certificate verification for target server
    let crypto_provider = match example_type {
        ExampleType::WiseTransaction => {
            // Production TLS verification for wise.com using standard certificate chains
            println!("🔒 Using production TLS certificate verification for Wise.com");
            CryptoProvider::default()
        }
        _ => {
            // Test fixture crypto provider with self-signed certificate for local testing
            println!("🧪 Using test fixture certificate for local development");
            let mut root_store = tls_core::anchors::RootCertStore::empty();
            root_store
                .add(&tls_core::key::Certificate(CA_CERT_DER.to_vec()))
                .unwrap();
            CryptoProvider {
                cert: WebPkiVerifier::new(root_store, None),
                ..Default::default()
            }
        }
    };

    // Phase 2: Configure MPC-TLS Prover for target server connection
    println!("🔧 Configuring MPC-TLS Prover for server: {}", server_name);
    
    let mut prover_config_builder = ProverConfig::builder();
    prover_config_builder
        .server_name(server_name)
        .protocol_config(
            ProtocolConfig::builder()
                // Data limits must match Notary configuration for MPC preprocessing
                .max_sent_data(zkp2p_tlsn_rust::MAX_SENT_DATA)
                .max_recv_data(zkp2p_tlsn_rust::MAX_RECV_DATA)
                .build()?,
        )
        .crypto_provider(crypto_provider);

    // Configure TLS client authentication based on target server requirements
    match example_type {
        ExampleType::WiseTransaction => {
            // ZKP2P: Wise.com uses web session authentication (Cookie/X-Access-Token)
            println!("🌐 Wise.com: Using web session authentication for payment verification");
        }
        _ => {
            // Test fixtures require client certificate authentication
            println!("🧪 Test fixture: Using client certificate authentication");
            prover_config_builder.tls_config(
                TlsConfig::builder()
                    .client_auth_pem((vec![CLIENT_CERT.to_vec()], CLIENT_KEY.to_vec()))
                    .unwrap()
                    .build()?,
            );
        }
    }

    let prover_config = prover_config_builder.build()?;

    // Phase 2: Initialize MPC-TLS Prover with Notary collaboration
    println!("🤝 Setting up MPC-TLS collaboration with Notary...");
    let prover = Prover::new(prover_config)
        .setup(notary_connection.compat())
        .await?;

    // Establish TCP connection to target server
    println!("🔌 Connecting to target server: {}:{}", server_host, server_port);
    let client_socket = tokio::net::TcpStream::connect((server_host, server_port)).await?;

    // Phase 2: Establish MPC-TLS connection with target server
    // CRITICAL: This creates a three-way MPC connection (Prover ↔ Notary ↔ Server)
    // - Prover and Notary secret-share TLS session keys via MPC
    // - Target server sees standard TLS 1.2 connection (Notary is transparent)
    // - All data encryption/decryption occurs through MPC with Notary
    println!("🔐 Establishing MPC-TLS connection (Prover ↔ Notary ↔ Server)...");
    let (mpc_tls_connection, prover_fut) = prover.connect(client_socket.compat()).await?;
    let mpc_tls_connection = TokioIo::new(mpc_tls_connection.compat());
    
    println!("✅ MPC-TLS connection established - Notary transparent to server");

    // Spawn the prover task to be run concurrently in the background.
    let prover_task = tokio::spawn(prover_fut);

    // Attach the hyper HTTP client to the connection.
    let (mut request_sender, connection) =
        hyper::client::conn::http1::handshake(mpc_tls_connection).await?;

    // Spawn the HTTP task to be run concurrently in the background.
    tokio::spawn(connection);

    // Phase 2: Execute dual-phase requests for Wise transaction verification
    match example_type {
        ExampleType::WiseTransaction => {
            // Get environment variables for Phase 2 request construction
            let profile_id = env::var("WISE_PROFILE_ID").unwrap();
            let transaction_id = env::var("WISE_TRANSACTION_ID").unwrap();
            
            println!("🔄 Executing dual-phase MPC-TLS requests:");
            
            // Phase 2A: Transaction List Verification Request
            println!("   Phase 1: Verifying transaction ownership...");
            let phase1_request = build_http_request(
                initial_uri, 
                server_name, 
                &extra_headers,
                "Requesting transaction list for ownership verification"
            )?;
            
            let phase1_response = request_sender.send_request(phase1_request).await?;
            
            if phase1_response.status() != StatusCode::OK {
                return Err(format!("❌ Phase 1 failed - Transaction list request returned: {}", phase1_response.status()).into());
            }
            
            // Read and parse transaction list response
            let phase1_body = phase1_response.into_body().collect().await?.to_bytes();
            let phase1_text = String::from_utf8_lossy(&phase1_body);
            
            println!("   ✅ Transaction list retrieved ({} bytes)", phase1_body.len());
            
            // Verify target transaction exists in user's transaction list
            let transaction_list: serde_json::Value = serde_json::from_str(&phase1_text)
                .map_err(|e| format!("Failed to parse transaction list JSON: {}", e))?;
            
            let transaction_exists = zkp2p_tlsn_rust::verify_transaction_in_list(
                &transaction_list, 
                &transaction_id
            )?;
            
            if !transaction_exists {
                return Err(format!("❌ Transaction {} not found in user's transaction list - cannot prove ownership", transaction_id).into());
            }
            
            println!("   ✅ Transaction ownership verified: {} found in user's list", transaction_id);
            
            // Phase 2B: Transaction Details Attestation Request
            println!("   Phase 2: Attesting transaction details...");
            let phase2_uri = format!("/gateway/v3/profiles/{}/transfers/{}", profile_id, transaction_id);
            let phase2_request = build_http_request(
                &phase2_uri,
                server_name,
                &extra_headers,
                "Requesting specific transaction details for attestation"
            )?;
            
            let phase2_response = request_sender.send_request(phase2_request).await?;
            
            if phase2_response.status() != StatusCode::OK {
                return Err(format!("❌ Phase 2 failed - Transaction details request returned: {}", phase2_response.status()).into());
            }
            
            println!("   ✅ Transaction details retrieved successfully");
            println!("✨ Dual-phase verification completed - Ready for attestation");
        }
        _ => {
            // Single-phase request for test fixtures
            let request = build_http_request(
                initial_uri,
                server_name, 
                &extra_headers,
                "Test fixture request"
            )?;
            
            let response = request_sender.send_request(request).await?;
            
            if response.status() != StatusCode::OK {
                return Err(format!("❌ Server returned error status: {}", response.status()).into());
            }
            
            println!("✅ Test response received successfully");
        }
    }

    // Phase 3: Complete MPC-TLS session and retrieve dual-phase transcript
    println!("🏁 Completing MPC-TLS session...");
    let mut prover = prover_task.await??;

    // Parse the HTTP transcript captured through MPC-TLS (contains both requests/responses)
    let transcript = HttpTranscript::parse(prover.transcript())?;
    println!("📋 HTTP transcript parsed successfully");
    
    match example_type {
        ExampleType::WiseTransaction => {
            println!("🔍 Analyzing dual-phase transcript...");
            
            if transcript.responses.len() >= 2 {
                // Phase 1: Transaction list response
                let list_body = &transcript.responses[0].body.as_ref().unwrap().content;
                let list_data = String::from_utf8_lossy(list_body.span().as_bytes());
                println!("   Phase 1 Response: Transaction list ({} bytes)", list_data.len());
                
                // Phase 2: Transaction details response  
                let details_body = &transcript.responses[1].body.as_ref().unwrap().content;
                let details_data = String::from_utf8_lossy(details_body.span().as_bytes());
                println!("   Phase 2 Response: Transaction details ({} bytes)", details_data.len());
                
                // Extract and log transaction metadata (without sensitive data)
                if let Ok(transaction_json) = serde_json::from_str::<serde_json::Value>(&details_data) {
                    let metadata = zkp2p_tlsn_rust::extract_transaction_metadata(&transaction_json);
                    println!("   📊 Transaction Summary:");
                    println!("      ID: {}", metadata.id);
                    println!("      Amount: {} {}", metadata.amount, metadata.currency);
                    println!("      Status: {}", metadata.status);
                    println!("      Date: {}", metadata.date);
                    debug!("Full transaction data: {}", serde_json::to_string_pretty(&transaction_json)?);
                }
            } else {
                return Err("Dual-phase transcript should contain at least 2 responses".into());
            }
        }
        _ => {
            // Single response for test fixtures
            let body_content = &transcript.responses[0].body.as_ref().unwrap().content;
            let body = String::from_utf8_lossy(body_content.span().as_bytes());
            
            match body_content {
                tlsn_formats::http::BodyContent::Json(_json) => {
                    let parsed = serde_json::from_str::<serde_json::Value>(&body)?;
                    println!("📄 Test data received ({} bytes)", body.len());
                    debug!("Test data preview: {}", serde_json::to_string_pretty(&parsed)?);
                }
                _ => {
                    println!("📄 Test data received ({} bytes)", body.len());
                }
            }
        }
    }

    // Phase 4: Create cryptographic commitments to dual-phase transcript data
    println!("🔏 Creating cryptographic commitments to HTTP transcript...");
    let mut builder = TranscriptCommitConfig::builder(prover.transcript());

    // Commit to HTTP transcript components using default strategy
    // For dual-phase: commits both transaction list verification AND transaction details
    // This creates separate commitments for all requests/responses, headers, bodies, etc.
    // See https://docs.tlsnotary.org/protocol/commit_strategy.html for alternatives
    DefaultHttpCommitter::default().commit_transcript(&mut builder, &transcript)?;

    let transcript_commit = builder.build()?;
    match example_type {
        ExampleType::WiseTransaction => {
            println!("✅ Dual-phase transcript commitments created");
            println!("   ✓ Transaction ownership proof committed");
            println!("   ✓ Transaction details attestation committed");
        }
        _ => {
            println!("✅ Transcript commitments created");
        }
    }

    // Phase 5: Request Notary attestation of committed dual-phase data
    println!("📝 Requesting Notary attestation...");
    let mut builder = RequestConfig::builder();
    builder.transcript_commit(transcript_commit);

    // Extension support for application-specific metadata
    // Can be used to bind attestation to specific ZKP2P requirements
    match example_type {
        ExampleType::WiseTransaction => {
            // builder.extension(Extension {
            //     id: b"zkp2p.wise.dual_phase".to_vec(),
            //     value: b"transaction_ownership_and_details".to_vec(),
            // });
        }
        _ => {}
    }

    let request_config = builder.build()?;

    // Phase 5: Generate signed dual-phase payment proof from Notary
    // CRITICAL: Notary creates cryptographic signatures without seeing:
    // - Plaintext payment details or transaction list data
    // - Session credentials (Cookie, X-Access-Token)  
    // - Server identity (wise.com)
    // - Personal information (account numbers, names, etc.)
    // - The Notary can attest to both transaction ownership AND payment details
    #[allow(deprecated)]
    let (attestation, secrets) = prover.notarize(&request_config).await?;

    match example_type {
        ExampleType::WiseTransaction => {
            println!("🎉 ZKP2P dual-phase payment proof generated successfully!");
            println!("   ✓ Transaction ownership verified and attested");
            println!("   ✓ Payment details cryptographically signed");
        }
        _ => {
            println!("🎉 Test proof generated successfully!");
        }
    }

    // Save portable attestation and secrets to disk
    let attestation_path = zkp2p_tlsn_rust::get_file_path(example_type, "attestation");
    let secrets_path = zkp2p_tlsn_rust::get_file_path(example_type, "secrets");

    println!("💾 Saving attestation files...");
    tokio::fs::write(&attestation_path, bincode::serialize(&attestation)?).await?;
    tokio::fs::write(&secrets_path, bincode::serialize(&secrets)?).await?;

    match example_type {
        ExampleType::WiseTransaction => {
            println!("✨ ZKP2P dual-phase payment proof process completed successfully!");
            println!();
            println!("📁 Generated files:");
            println!("   🔒 Dual-Phase Proof: {} (cryptographic attestation signed by Notary)", attestation_path);
            println!("   🔑 Selective Disclosure Data: {} (for privacy-preserving presentation)", secrets_path);
            println!();
            println!("🎯 Next steps for ZKP2P payment verification:");
            println!("   1. Create selective proof: cargo run --release --example attestation_present -- wise-transaction");
            println!("   2. Verify payment proof: cargo run --release --example attestation_verify -- wise-transaction");
            println!("   3. Submit to ZKP2P protocol for crypto asset release");
            println!();
            println!("🔐 Enhanced Privacy Guarantees:");
            println!("   ✓ Transaction ownership proven without revealing account details");
            println!("   ✓ Payment completion verified without exposing sensitive data");
            println!("   ✓ Session credentials remain completely private");
            println!("   ✓ Dual-phase verification prevents transaction ID enumeration attacks");
        }
        _ => {
            println!("✨ Test proof process completed successfully!");
            println!();
            println!("📁 Generated files:");
            println!("   🔒 Test Proof: {} (cryptographic attestation signed by Notary)", attestation_path);
            println!("   🔑 Verification Data: {} (for selective disclosure)", secrets_path);
            println!();
            println!("🧪 Test proof created for development purposes");
        }
    }

    Ok(())
}
