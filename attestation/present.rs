use clap::Parser;
use hyper::header;

use tlsn_core::{CryptoProvider, Secrets, attestation::Attestation, presentation::Presentation};
use tlsn_formats::http::HttpTranscript;
use zkp2p_tlsn_rust::ExampleType;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// What data to notarize
    #[clap(default_value_t, value_enum)]
    example_type: ExampleType,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    create_presentation(&args.example_type).await
}

async fn create_presentation(example_type: &ExampleType) -> Result<(), Box<dyn std::error::Error>> {
    let attestation_path = zkp2p_tlsn_rust::get_file_path(example_type, "attestation");
    let secrets_path = zkp2p_tlsn_rust::get_file_path(example_type, "secrets");

    println!("🎭 Creating selective disclosure presentation for {:?}...", example_type);
    println!("   Reading attestation from: {}", attestation_path);
    println!("   Reading secrets from: {}", secrets_path);

    // Read attestation from disk.
    let attestation: Attestation = bincode::deserialize(&std::fs::read(attestation_path)?)?;

    // Read secrets from disk.
    let secrets: Secrets = bincode::deserialize(&std::fs::read(secrets_path)?)?;

    // Parse the HTTP transcript (may contain multiple requests/responses for dual-phase)
    let transcript = HttpTranscript::parse(secrets.transcript())?;
    
    match example_type {
        ExampleType::WiseTransaction => {
            println!("🔍 Analyzing dual-phase transcript for selective disclosure...");
            println!("   Requests: {}", transcript.requests.len());
            println!("   Responses: {}", transcript.responses.len());
            
            if transcript.requests.len() < 2 || transcript.responses.len() < 2 {
                return Err("Dual-phase attestation should contain at least 2 requests and 2 responses".into());
            }
        }
        _ => {
            println!("📋 Processing single-phase transcript...");
        }
    }

    // Build a transcript proof with selective disclosure
    let mut builder = secrets.transcript_proof_builder();

    // Selective disclosure strategy based on example type
    match example_type {
        ExampleType::WiseTransaction => {
            println!("🎯 Applying ZKP2P dual-phase selective disclosure strategy...");
            
            // Phase 1: Transaction List Request - Reveal structure but hide sensitive headers
            let list_request = &transcript.requests[0];
            println!("   Phase 1 Request: Revealing transaction list request structure...");
            builder.reveal_sent(&list_request.without_data())?;
            builder.reveal_sent(&list_request.request.target)?;
            
            // Phase 1: Headers - Hide authentication credentials but show request structure
            for header in &list_request.headers {
                let header_name = header.name.as_str();
                let should_hide_value = header_name.eq_ignore_ascii_case(header::USER_AGENT.as_str())
                    || header_name.eq_ignore_ascii_case("Cookie")
                    || header_name.eq_ignore_ascii_case("X-Access-Token");
                
                if should_hide_value {
                    builder.reveal_sent(&header.without_value())?;
                    println!("     Header {}: [HIDDEN]", header_name);
                } else {
                    builder.reveal_sent(header)?;
                    println!("     Header {}: revealed", header_name);
                }
            }
            
            // Phase 2: Transaction Details Request - Reveal structure but hide sensitive headers
            let details_request = &transcript.requests[1];
            println!("   Phase 2 Request: Revealing transaction details request structure...");
            builder.reveal_sent(&details_request.without_data())?;
            builder.reveal_sent(&details_request.request.target)?;
            
            // Phase 2: Headers - Hide authentication credentials
            for header in &details_request.headers {
                let header_name = header.name.as_str();
                let should_hide_value = header_name.eq_ignore_ascii_case(header::USER_AGENT.as_str())
                    || header_name.eq_ignore_ascii_case("Cookie")
                    || header_name.eq_ignore_ascii_case("X-Access-Token");
                
                if should_hide_value {
                    builder.reveal_sent(&header.without_value())?;
                    println!("     Header {}: [HIDDEN]", header_name);
                } else {
                    builder.reveal_sent(header)?;
                    println!("     Header {}: revealed", header_name);
                }
            }
        }
        _ => {
            // Single-phase request for test fixtures
            let request = &transcript.requests[0];
            builder.reveal_sent(&request.without_data())?;
            builder.reveal_sent(&request.request.target)?;
            
            for header in &request.headers {
                let header_name = header.name.as_str();
                let should_hide_value = header_name.eq_ignore_ascii_case(header::USER_AGENT.as_str())
                    || header_name.eq_ignore_ascii_case(header::AUTHORIZATION.as_str());
                
                if should_hide_value {
                    builder.reveal_sent(&header.without_value())?;
                } else {
                    builder.reveal_sent(header)?;
                }
            }
        }
    }

    // Response selective disclosure based on example type
    match example_type {
        ExampleType::WiseTransaction => {
            println!("📊 Processing dual-phase responses for ZKP2P verification...");
            
            // Phase 1: Transaction List Response - Reveal structure and ownership proof
            let list_response = &transcript.responses[0];
            println!("   Phase 1 Response: Transaction list ownership proof...");
            builder.reveal_recv(&list_response.without_data())?;
            
            // Reveal response headers for list request
            for header in &list_response.headers {
                builder.reveal_recv(header)?;
            }
            
            // For transaction list, we can reveal the structure to prove ownership
            // but we don't need to reveal individual transaction details except for existence proof
            let list_content = &list_response.body.as_ref().unwrap().content;
            if let tlsn_formats::http::BodyContent::Json(list_json) = list_content {
                // Reveal metadata to prove this is a legitimate transaction list
                if let Some(meta) = list_json.get("meta") {
                    if let Some(count) = meta.get("totalCount") {
                        builder.reveal_recv(count)?;
                        println!("     Revealed: Transaction count for ownership proof");
                    }
                }
                
                // For ownership proof, we reveal the existence of our target transaction
                // without revealing details of other transactions
                if let Some(data) = list_json.get("data") {
                    let target_transaction_id = std::env::var("WISE_TRANSACTION_ID").unwrap_or_default();
                    println!("     Searching for target transaction ID: {}", target_transaction_id);
                    
                    // For arrays in JsonValue, we need to work with indices
                    // Try to parse as serde_json::Value temporarily to find the target transaction
                    let data_str = serde_json::to_string(data)?;
                    let data_json: serde_json::Value = serde_json::from_str(&data_str)?;
                    
                    if let Some(transactions) = data_json.as_array() {
                        for (index, transaction) in transactions.iter().enumerate() {
                            let mut found_target = false;
                            
                            // Check both id and resource.id formats
                            if let Some(id) = transaction.get("id").and_then(|id| id.as_str()) {
                                if id == target_transaction_id {
                                    found_target = true;
                                }
                            }
                            if let Some(resource) = transaction.get("resource") {
                                if let Some(id) = resource.get("id").and_then(|id| id.as_str()) {
                                    if id == target_transaction_id {
                                        found_target = true;
                                    }
                                }
                            }
                            
                            if found_target {
                                // Reveal the transaction ID from the original JsonValue
                                if let Some(transaction_obj) = data.get(&index.to_string()) {
                                    if let Some(id_field) = transaction_obj.get("id") {
                                        builder.reveal_recv(id_field)?;
                                        println!("     Revealed: Target transaction ID {} for ownership proof", target_transaction_id);
                                    }
                                    if let Some(resource) = transaction_obj.get("resource") {
                                        if let Some(id_field) = resource.get("id") {
                                            builder.reveal_recv(id_field)?;
                                            println!("     Revealed: Target transaction resource.id {} for ownership proof", target_transaction_id);
                                        }
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
            
            // Phase 2: Transaction Details Response - Reveal essential payment data for ZKP2P
            let details_response = &transcript.responses[1];
            println!("   Phase 2 Response: Transaction details for payment verification...");
            builder.reveal_recv(&details_response.without_data())?;
            
            // Reveal response headers for details request
            for header in &details_response.headers {
                builder.reveal_recv(header)?;
            }
            
            let details_content = &details_response.body.as_ref().unwrap().content;
            if let tlsn_formats::http::BodyContent::Json(details_json) = details_content {
                // ZKP2P Payment Verification: Reveal only essential fields for payment proof
                println!("     Applying ZKP2P selective disclosure strategy...");
                
                // Payment amount - critical for verifying correct payment value
                if let Some(amount) = details_json.get("primaryAmount") {
                    builder.reveal_recv(amount)?;
                    println!("     Revealed: primaryAmount for payment verification");
                }
                
                // Payment ID - unique identifier for linking to specific payment order
                if let Some(resource) = details_json.get("resource") {
                    if let Some(id) = resource.get("id") {
                        builder.reveal_recv(id)?;
                        println!("     Revealed: resource.id for payment identification");
                    }
                }
                
                // Recipient identifier - proves payment went to correct seller
                if let Some(title) = details_json.get("title") {
                    builder.reveal_recv(title)?;
                    println!("     Revealed: title for recipient verification");
                }
                
                // Payment timestamp - validates payment occurred within order timeframe
                if let Some(date) = details_json.get("visibleOn") {
                    builder.reveal_recv(date)?;
                    println!("     Revealed: visibleOn for timing verification");
                }
                
                // Currency - ensures payment was made in correct currency
                if let Some(currency) = details_json.get("currency") {
                    builder.reveal_recv(currency)?;
                    println!("     Revealed: currency for payment verification");
                }
                
                // Payment status - critical proof that payment completed successfully
                if let Some(status) = details_json.get("status") {
                    builder.reveal_recv(status)?;
                    println!("     Revealed: status for completion verification");
                }
                
                println!("   ✅ ZKP2P essential fields revealed, sensitive data remains private");
            }
        }
        _ => {
            // Single response for test fixtures
            let response = &transcript.responses[0];
            builder.reveal_recv(&response.without_data())?;
            
            for header in &response.headers {
                builder.reveal_recv(header)?;
            }
            
            let content = &response.body.as_ref().unwrap().content;
            match content {
                tlsn_formats::http::BodyContent::Json(json) => {
                    // For test fixtures, reveal selected fields
                    let reveal_all = false;
                    if reveal_all {
                        builder.reveal_recv(response)?;
                    } else {
                        builder.reveal_recv(json.get("id").unwrap())?;
                        builder.reveal_recv(json.get("information.name").unwrap())?;
                        builder.reveal_recv(json.get("meta.version").unwrap())?;
                    }
                }
                tlsn_formats::http::BodyContent::Unknown(span) => {
                    builder.reveal_recv(span)?;
                }
                _ => {}
            }
        }
    }

    let transcript_proof = builder.build()?;

    // Use default crypto provider to build the presentation.
    let provider = CryptoProvider::default();

    let mut builder = attestation.presentation_builder(&provider);

    builder
        .identity_proof(secrets.identity_proof())
        .transcript_proof(transcript_proof);

    let presentation: Presentation = builder.build()?;

    let presentation_path = zkp2p_tlsn_rust::get_file_path(example_type, "presentation");

    // Write the presentation to disk.
    std::fs::write(&presentation_path, bincode::serialize(&presentation)?)?;

    match example_type {
        ExampleType::WiseTransaction => {
            println!("🎉 ZKP2P dual-phase presentation built successfully!");
            println!("   ✓ Transaction ownership proof with privacy preservation");
            println!("   ✓ Payment details with selective disclosure");
            println!("   ✓ Authentication credentials completely hidden");
            println!();
            println!("📁 Presentation written to: {}", presentation_path);
            println!();
            println!("🔒 Privacy Summary:");
            println!("   • Session cookies and tokens: HIDDEN");
            println!("   • Account numbers and personal details: HIDDEN");
            println!("   • Other transactions in list: HIDDEN");
            println!("   • Payment amount, currency, status: REVEALED for verification");
            println!("   • Transaction ID and date: REVEALED for matching");
        }
        _ => {
            println!("🎉 Test presentation built successfully!");
            println!("📁 The presentation has been written to: {}", presentation_path);
        }
    }

    println!();
    println!("🎯 Next step: Run verification with:");
    println!("   cargo run --release --example attestation_verify -- {:?}", example_type.to_string().to_ascii_lowercase());
    
    Ok(())
}
