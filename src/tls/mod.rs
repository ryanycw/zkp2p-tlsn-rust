use crate::ExampleType;
use tls_core::verify::WebPkiVerifier;
use tls_server_fixture::CA_CERT_DER;
use tlsn_common::config::ProtocolConfig;
use tlsn_core::CryptoProvider;
use tlsn_prover::{Prover, ProverConfig, TlsConfig};
use tlsn_server_fixture_certs::{CLIENT_CERT, CLIENT_KEY};

/// Creates a crypto provider based on the example type
pub fn create_crypto_provider(example_type: &ExampleType) -> CryptoProvider {
    match example_type {
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
    }
}

/// Builds prover configuration for MPC-TLS session
pub fn build_prover_config(
    server_name: &str,
    max_sent: usize,
    max_recv: usize,
    crypto_provider: CryptoProvider,
    example_type: &ExampleType,
) -> Result<ProverConfig, Box<dyn std::error::Error>> {
    println!("🔧 Configuring MPC-TLS Prover for server: {}", server_name);

    let mut builder = ProverConfig::builder();
    builder
        .server_name(server_name)
        .protocol_config(
            ProtocolConfig::builder()
                .max_sent_data(max_sent)
                .max_recv_data(max_recv)
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
            builder.tls_config(
                TlsConfig::builder()
                    .client_auth_pem((vec![CLIENT_CERT.to_vec()], CLIENT_KEY.to_vec()))
                    .unwrap()
                    .build()?,
            );
        }
    }

    Ok(builder.build()?)
}

/// Sets up MPC-TLS prover with notary collaboration
pub async fn setup_mpc_tls_prover<T>(
    prover_config: ProverConfig,
    notary_connection: T,
) -> Result<tlsn_prover::Prover<tlsn_prover::state::Setup>, Box<dyn std::error::Error>>
where
    T: futures::AsyncRead + futures::AsyncWrite + Send + Unpin + 'static,
{
    println!("🤝 Setting up MPC-TLS collaboration with Notary...");

    let prover = Prover::new(prover_config).setup(notary_connection).await?;

    Ok(prover)
}

/// Establishes MPC-TLS connection with target server
pub async fn connect_to_server(
    server_host: &str,
    server_port: u16,
) -> Result<tokio::net::TcpStream, Box<dyn std::error::Error>> {
    println!(
        "🔌 Connecting to target server: {}:{}",
        server_host, server_port
    );
    let socket = tokio::net::TcpStream::connect((server_host, server_port)).await?;
    Ok(socket)
}
