# CLAUDE.md

Development guide for zkp2p-tlsn-rust.

## Project Overview

ZKP2P payment verification using TLSNotary - a working CLI tool that proves Wise.com transaction completion cryptographically without revealing sensitive credentials.

**What it does**: Generates cryptographic proofs of fiat payments for ZKP2P settlements using the Ethereum Foundation's TLSNotary protocol.

## Project Structure

```
src/
├── config.rs              # Configuration management from environment
├── domain/                 # Core business logic
│   ├── args.rs            # CLI argument definitions (clap)
│   ├── providers.rs       # Provider configurations (Wise, PayPal)
│   ├── server.rs          # Server connection configurations
│   └── transaction.rs     # Transaction data structures
└── utils/                  # Utilities
    ├── file_io.rs         # File operations for attestations
    ├── info.rs            # Logging and tracing setup
    ├── notary.rs          # Notary server communication
    ├── providers.rs       # Provider-specific request handling
    ├── text_parser.rs     # HTTP response parsing
    └── tls.rs             # MPC-TLS configuration

attestation/                # Main binaries
├── prove.rs               # Main prover binary (zkp2p-prove)
└── verify.rs              # Verification binary (zkp2p-verify)
```

## Binaries

- `zkp2p-prove` - Generate proofs and presentations
- `zkp2p-verify` - Verify proofs

## Development Commands

```bash
# Build
cargo build --release

# Run prover
cargo run --release --bin zkp2p-prove -- --help

# Run verifier  
cargo run --release --bin zkp2p-verify -- --help

# Test
cargo test
```

## CLI Interface

The prover supports multiple modes and providers:

```bash
# Generate proof
zkp2p-prove --mode prove --provider wise \
  --profile-id "123" --transaction-id "456" \
  --cookie "session..." --access-token "token..."

# Create presentation
zkp2p-prove --mode present --provider wise

# Do both
zkp2p-prove --mode prove-to-present --provider wise \
  --profile-id "123" --transaction-id "456" \
  --cookie "session..." --access-token "token..."
```

## Configuration

Environment variables in `.env`:

```bash
NOTARY_HOST=notary.pse.dev    # or 127.0.0.1 for local
NOTARY_PORT=7047
NOTARY_TLS=true               # or false for local testing
WISE_HOST=wise.com
WISE_PORT=443
MAX_SENT_DATA=4096
MAX_RECV_DATA=65536
USER_AGENT="Mozilla/5.0 ..."
```

## TLSNotary Flow

1. **MPC-TLS Setup**: Prover and Notary share TLS session keys
2. **HTTP Request**: Prover makes authenticated request to Wise.com
3. **Attestation**: Notary signs cryptographic commitments to the data
4. **Presentation**: Prover creates selective disclosure of payment fields
5. **Verification**: Anyone can verify the proof using Notary's signature

## Key Components

### Providers (`utils/providers.rs`)
- Handles different payment providers (Wise, PayPal)
- Builds HTTP requests with proper authentication
- Extracts payment data from responses

### Text Parser (`utils/text_parser.rs`)
- Parses HTTP responses to find payment fields
- Extracts ranges for selective disclosure
- Provider-specific field mapping

### File I/O (`utils/file_io.rs`)
- Saves/loads attestations, secrets, presentations
- Handles file naming and paths
- Serialization with bincode

## Adding New Providers

1. Add variant to `Provider` enum in `args.rs`
2. Add configuration in `providers.rs`
3. Implement request building in `utils/providers.rs`
4. Add field parsing patterns in `utils/text_parser.rs`

## Local Testing

```bash
# Terminal 1 - Start local notary
cd /path/to/tlsn
cargo run --release --bin notary-server

# Terminal 2 - Configure for local testing
cp .env.local .env
cargo run --release --bin zkp2p-prove -- --mode prove-to-present --provider wise \
  --profile-id "your_id" --transaction-id "your_tx" \
  --cookie "your_cookie" --access-token "your_token"
```

## Dependencies

- **TLSNotary v0.1.0-alpha.12**: Core protocol implementation
- **clap**: CLI argument parsing
- **tokio**: Async runtime
- **hyper**: HTTP client
- **serde**: Serialization
- **tracing**: Logging

## Current Status

- ✅ Working CLI binaries
- ✅ Wise.com integration 
- ✅ PayPal provider structure (implementation pending)
- ✅ Local and production notary support
- ✅ Comprehensive error handling
- ✅ Modular architecture