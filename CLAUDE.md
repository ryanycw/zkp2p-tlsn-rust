# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust implementation of ZKP2P payment verification using TLSNotary, built on the TLSNotary protocol developed by the Ethereum Foundation's Privacy and Scaling Explorations (PSE) team. **The primary goal is to create a production-ready binary prover that generates cryptographic proofs of payment completion through Wise.com. This binary will be distributed to the [ZKP2P React Native SDK](https://github.com/zkp2p/zkp2p-react-native-sdk) repository for mobile integration.**

**Key Innovation**: This implementation uses a **dual-phase attestation architecture** that first proves transaction ownership through the user's transaction list, then attests specific payment details. This approach prevents transaction ID enumeration attacks while maintaining complete privacy of authentication credentials and sensitive financial data.

## Binary Distribution Strategy

### Project Scope
This repository focuses exclusively on creating a production-ready binary prover. The React Native SDK integration, FFI bridge, and mobile-specific optimizations will be handled separately in the [zkp2p-react-native-sdk](https://github.com/zkp2p/zkp2p-react-native-sdk) repository.

### Binary Requirements

1. **Production CLI**: Convert current examples into a standalone binary with comprehensive CLI interface
2. **Cross-Platform Compilation**: Support compilation for multiple targets (iOS, Android, desktop)
3. **Optimized Performance**: Memory and processing optimizations suitable for mobile deployment
4. **Robust Error Handling**: Clear error codes and messages for external integration
5. **JSON I/O**: JSON-based input/output for easy integration with other systems
6. **File-Based Operations**: Self-contained operations using file system for input/output

## TLSNotary Protocol Architecture

TLSNotary is a three-party protocol involving:

### 1. **Prover (Client)** 
- Initiates connection with target server (e.g., wise.com)
- Collaborates with Notary using MPC to perform TLS operations
- Maintains privacy of plaintext data while proving its authenticity
- Creates selective disclosure presentations from attestations

### 2. **Notary Server**
- Acts as a trusted third-party verifier during TLS connection
- Participates in MPC-TLS without seeing actual data content
- Cryptographically signs commitments to data authenticity
- Enables portable and reusable proofs
- **Critical**: Server (wise.com) sees only standard TLS - Notary is transparent

### 3. **Verifier**
- Validates cryptographic proofs without trusting the Prover
- Confirms data authenticity using Notary's cryptographic signature
- Can verify selective disclosures while maintaining privacy guarantees

## MPC-TLS Protocol Flow

### Phase 1: MPC-TLS Connection
1. **Handshake**: Prover initiates TLS 1.2 handshake with target server
2. **Key Sharing**: Prover and Notary secret-share TLS session keys via MPC
3. **Transparent Operation**: Target server sees normal TLS connection
4. **Data Exchange**: Encrypted communication occurs with cryptographic guarantees

### Phase 2: Notarization  
1. **Commitment**: Notary creates cryptographic commitments to data and server identity
2. **Signing**: Notary signs attestation making it portable and verifiable
3. **Privacy**: Notary never sees plaintext data or server identity

### Phase 3: Selective Disclosure
1. **Redaction**: Prover selectively reveals only necessary data fields
2. **Zero-Knowledge**: Can prove properties of data without revealing content
3. **Presentation**: Creates verifiable proof with controlled information disclosure

## Code Components

### Modular Architecture

The codebase follows a modular design pattern with clear separation of concerns:

#### Core Modules (`src/`)

- **`attestation/`**: Attestation operations and file handling
  - `analyze_transcript()`: Processes and logs transcript data
  - `create_transcript_commitment()`: Creates cryptographic commitments
  - `notarize_transcript()`: Requests notary attestation
  - `save_attestation_files()`: Persists attestation to disk

- **`http/`**: HTTP request building utilities
  - `build_request()`: Constructs HTTP requests with common headers
  - Handles header redaction for sensitive data

- **`notary/`**: Notary server configuration and connection
  - `NotaryConfig`: Configuration management from environment
  - `request_notarization()`: Handles notarization request flow

- **`providers/`**: Provider-specific implementations
  - `wise.rs`: Wise.com specific logic and dual-phase execution
  - `ServerConfig`: Generic server configuration

- **`tls/`**: MPC-TLS session management
  - `create_crypto_provider()`: TLS certificate verification setup
  - `build_prover_config()`: Prover configuration
  - `setup_mpc_tls_prover()`: MPC-TLS initialization

#### Example Implementations (`attestation/`)

- **Prover** (`prove.rs`): Orchestrates dual-phase MPC-TLS protocol:
  - **Phase 1**: Verifies transaction ownership via transaction list
  - **Phase 2**: Attests specific payment details
  - Coordinates modules for end-to-end attestation flow
  
- **Presenter** (`present.rs`): Creates selective disclosure presentations:
  - **Transaction List Disclosure**: Reveals only target transaction ID
  - **Payment Details Disclosure**: Reveals essential ZKP2P fields
  - **Privacy Preservation**: Hides sensitive credentials and data
  
- **Verifier** (`verify.rs`): Validates dual-phase presentations:
  - **Ownership Verification**: Confirms transaction authenticity
  - **Payment Verification**: Validates payment details
  - **Cryptographic Integrity**: Ensures attestation validity

### Key Dependencies

The project uses TLSNotary v0.1.0-alpha.12 with these core crates:
- `tlsn-prover`: Client-side proving functionality
- `tlsn-verifier`: Verification of presentations  
- `notary-client`: Connection to notary servers
- `tlsn-formats`: HTTP transcript parsing and commitment strategies

## Development Commands

### Building
```bash
cargo build                    # Debug build
cargo build --release         # Release build (recommended for examples)
cargo check                    # Type checking without compilation
```

### Running Examples
The project uses Cargo examples rather than binaries:

```bash
# Generate Wise transaction attestation (requires manual credential extraction)
cargo run --release --example attestation_prove -- wise-transaction \
  --wise-profile-id "12345" --wise-transaction-id "67890" \
  --wise-cookie "session..." --wise-access-token "token..."

# Create a presentation from attestation
cargo run --release --example attestation_present -- wise-transaction

# Verify a presentation  
cargo run --release --example attestation_verify -- wise-transaction

# Test fixtures (development only)
cargo run --release --example attestation_prove
```

### Testing Setup

The examples require running TLSNotary infrastructure locally:

1. **Test Server** (terminal 1):
```bash
# From tlsnotary/tlsn repository
RUST_LOG=info PORT=4000 cargo run --bin tlsn-server-fixture
```

2. **Notary Server** (terminal 2):
```bash
# From tlsnotary/tlsn repository  
cargo run --release --bin notary-server
```

3. **Prover** (terminal 3):
```bash
# From this repository
SERVER_PORT=4000 cargo run --release --example attestation_prove
```

### Environment Variables

**General Configuration:**
- `NOTARY_HOST`: Notary server host (default: 127.0.0.1)
- `NOTARY_PORT`: Notary server port (default: 7047)
- `NOTARY_TLS`: Enable TLS for notary connection (default: true for WiseTransaction)

**Test Fixture Configuration:**
- `SERVER_HOST`: Target server host (default: 127.0.0.1)
- `SERVER_PORT`: Target server port (default: fixture server port)

**Wise.com Configuration:**
- `WISE_HOST`: Wise web interface host (default: wise.com)
- `WISE_PORT`: Wise web interface port (default: 443)
- `WISE_EMAIL`: Email for automated authentication (optional)
- `WISE_PASSWORD`: Password for automated authentication (optional)

### Testing
```bash
cargo test              # Run unit tests
cargo test --release    # Run tests in release mode
```

## File Structure

```
src/
├── lib.rs                 # Core constants and utilities (MAX_SENT_DATA, MAX_RECV_DATA, ExampleType)
├── attestation/           # Attestation operations and file handling
│   └── mod.rs            # Transcript analysis, commitment creation, notarization
├── http/                  # HTTP request building
│   └── mod.rs            # Request construction with headers
├── notary/                # Notary server interaction
│   └── mod.rs            # Notary client configuration and connection
├── providers/             # Provider-specific logic
│   ├── mod.rs            # Server configuration base
│   └── wise.rs           # Wise.com specific implementation
└── tls/                   # MPC-TLS session management
    └── mod.rs            # TLS configuration and prover setup

attestation/               # Example implementations
├── prove.rs              # Dual-phase MPC-TLS orchestration
├── present.rs            # Selective disclosure presentation creation
└── verify.rs             # Presentation verification

Generated files:
- `example-*.tlsn`: Attestation, secrets, and presentation files
```

## TLSNotary Configuration Notes

- **Protocol Support**: Currently supports TLS 1.2 (TLS 1.3 planned for future releases)
- **Data Limits**: Configure via `MAX_SENT_DATA` (4KB) and `MAX_RECV_DATA` (64KB) constants
- **MPC Preprocessing**: Protocol preprocesses MPC for sent data transcript to reduce connection times
- **Certificate Handling**: 
  - Test fixtures use self-signed certificates (`CA_CERT_DER`)
  - Production uses standard certificate validation (`CryptoProvider::default()`)
- **Compression**: Disabled (`Accept-Encoding: identity`) as TLSNotary doesn't support it
- **Notary Trust Model**: Verifiers can require signatures from multiple Notaries to prevent collusion

## Common Development Patterns

When modifying selective disclosure in `present.rs`:
- Use `builder.reveal_sent()` for request data
- Use `builder.reveal_recv()` for response data  
- Use `.without_data()` or `.without_value()` to hide sensitive parts
- JSON responses can be selectively revealed by field path (e.g., `json.get("meta.version")`)

When adding new example types, update the `ExampleType` enum in `src/lib.rs` and add corresponding URI mappings in `prove.rs`.

## ZKP2P Wise Payment Verification

### Purpose
This implementation enables buyers in ZKP2P protocol to cryptographically prove they completed fiat payments through Wise.com, allowing automated release of crypto assets from escrow without revealing sensitive financial information.

### Setup Required
1. Complete payment to seller through Wise.com
2. Manually extract credentials from your browser after logging into Wise
3. Identify the payment ID you want to prove completion for (from wise.com/all-transactions)

### ZKP2P Integration Flow
```bash
# Configure production notary
export NOTARY_HOST="notary.pse.dev"
export NOTARY_PORT="7047"
export NOTARY_TLS="true"

# 1. Extract credentials manually from browser (see README.md for detailed steps)
# 2. Generate payment proof with CLI arguments
cargo run --release --example attestation_prove -- wise-transaction \
  --wise-profile-id "your_profile_id" \
  --wise-transaction-id "your_payment_id" \
  --wise-cookie "your_session_cookie" \
  --wise-access-token "your_access_token"

# 3. Create selective payment presentation  
cargo run --release --example attestation_present -- wise-transaction

# 4. Verify payment proof (typically done by ZKP2P smart contract)
cargo run --release --example attestation_verify -- wise-transaction
```

### Payment Verification Data
The ZKP2P payment proof reveals only essential fields:
- `primaryAmount`: Payment amount (for value verification)
- `currency`: Payment currency (for order matching)
- `resource.id`: Payment ID (for linking to order)
- `title`: Recipient identifier (for seller verification)
- `visibleOn`: Payment timestamp (for timeframe validation)
- `status`: Payment completion status (proof of successful transfer)

**Privacy Guarantee**: Session credentials, account numbers, full names, and other sensitive data remain completely private.

## Binary CLI Interface Design

### Command-Line Interface

```bash
# Primary proving operation
zkp2p-tlsn-prover prove \
  --config "/path/to/config.json" \
  --output-dir "/path/to/output"

# Alternative: inline parameters
zkp2p-tlsn-prover prove \
  --profile-id "12345678" \
  --transaction-id "987654321" \
  --cookie "session_data..." \
  --access-token "token..." \
  --notary-host "notary.pse.dev" \
  --notary-port 7047 \
  --output-dir "/path/to/output"

# Presentation creation
zkp2p-tlsn-prover present \
  --attestation-file "/path/to/attestation.tlsn" \
  --secrets-file "/path/to/secrets.tlsn" \
  --output-file "/path/to/presentation.tlsn"

# Verification
zkp2p-tlsn-prover verify \
  --presentation-file "/path/to/presentation.tlsn" \
  --output-file "/path/to/verification-result.json"
```

### Configuration File Format

**JSON Configuration (`config.json`):**
```json
{
  "wise": {
    "profile_id": "12345678",
    "transaction_id": "987654321",
    "cookie": "session_data...",
    "access_token": "eyJhbGciOiJIUzI1NiI...",
    "host": "wise.com",
    "port": 443
  },
  "notary": {
    "host": "notary.pse.dev",
    "port": 7047,
    "tls_enabled": true
  },
  "output": {
    "format": "json",
    "attestation_file": "attestation.tlsn",
    "secrets_file": "secrets.tlsn",
    "presentation_file": "presentation.tlsn"
  }
}
```

### Binary Output Format

**Success Response:**
```json
{
  "status": "success",
  "operation": "prove",
  "files": {
    "attestation": "/path/to/attestation.tlsn",
    "secrets": "/path/to/secrets.tlsn"
  },
  "metadata": {
    "transaction_id": "987654321",
    "proof_generated_at": "2025-01-24T10:30:00Z",
    "duration_ms": 45000
  }
}
```

**Error Response:**
```json
{
  "status": "error",
  "error_code": "WISE_AUTH_FAILED",
  "message": "Authentication with Wise.com failed. Please check your session credentials.",
  "details": {
    "operation": "prove",
    "phase": "wise_authentication"
  }
}
```

## Local Testing Setup

### Testing with Real Wise Server + Local Notary

**Prerequisites:**
1. Clone TLSNotary repository: `git clone https://github.com/tlsnotary/tlsn.git`
2. Have active Wise account with recent transactions
3. Rust development environment

**Step 1: Start Local Notary Server**
```bash
# Terminal 1 - Start local notary
cd /path/to/tlsn
cargo run --release --bin notary-server

# Expected output: "Notary server listening on 127.0.0.1:7047"
```

**Step 2: Configure Environment**
```bash
# Terminal 2 - Configure test environment
cd /path/to/zkp2p-tlsn-rust
cp .env.local .env

# Manual credential extraction required - see README.md for steps
```

**Step 3: Run Manual Credential Extraction + Proving**
```bash
# Get transaction ID from wise.com/all-transactions?direction=OUTGOING
export TRANSACTION_ID="your_transaction_id"

# Extract credentials manually from browser (see README.md for detailed steps)
# Then run proving with your extracted credentials:
cargo run --release --example attestation_prove -- wise-transaction \
  --wise-profile-id "your_profile_id" \
  --wise-transaction-id $TRANSACTION_ID \
  --wise-cookie "your_cookie_header" \
  --wise-access-token "your_access_token"

# Create selective presentation
cargo run --release --example attestation_present -- wise-transaction

# Verify the proof
cargo run --release --example attestation_verify -- wise-transaction
```

**Expected Test Output:**
```
🚀 ZKP2P Wise Authentication & Proving Pipeline
🔐 Step 1: Extracting Wise credentials...
✅ Wise authentication successful!
📋 Extracted credentials:
   Profile ID: 12345678
   Cookie: session_id=abc123...
   Access Token: eyJhbGciOiJIUz...

🔐 Step 2: Running ZKP2P attestation proving...
🚀 Starting ZKP2P payment verification via TLSNotary...
📡 Connecting to Notary server: 127.0.0.1:7047
🌐 Target server: wise.com (production HTTPS)
✅ Notary connection established
🔄 Executing dual-phase MPC-TLS requests:
   Phase 1: Verifying transaction ownership...
   ✅ Transaction ownership verified: [ID] found in user's list
   Phase 2: Attesting transaction details...
   ✅ Transaction details retrieved successfully
🎉 ZKP2P dual-phase payment proof generated successfully!
```

### Cross-Platform Compilation

**Target Platforms for Binary Distribution:**
```bash
# iOS targets (for React Native integration)
rustup target add aarch64-apple-ios
rustup target add x86_64-apple-ios  # iOS Simulator

# Android targets (for React Native integration)
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add i686-linux-android
rustup target add x86_64-linux-android  # Android Emulator

# Desktop targets (for development/testing)
rustup target add x86_64-apple-darwin   # macOS
rustup target add x86_64-unknown-linux-gnu  # Linux
rustup target add x86_64-pc-windows-msvc    # Windows

# Build binaries for all targets
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-linux-android
cargo build --release  # Host platform
```

## Implementation Status

**✅ PHASE 1 COMPLETED - Dual-Phase TLSNotary Core**
- Dual-phase MPC-TLS implementation
- Transaction ownership verification + payment details attestation
- Enhanced security against enumeration attacks
- Complete privacy preservation

**🔄 PHASE 2 IN PROGRESS - Production Binary Development**
- Binary CLI interface implementation
- JSON-based configuration and output
- Cross-platform compilation support
- Performance optimization for mobile deployment
- Comprehensive error handling and logging

**📋 PHASE 3 PLANNED - Binary Distribution**
- Cross-platform binary builds (iOS, Android, Desktop)
- Binary distribution to zkp2p-react-native-sdk repository
- Integration testing with external systems
- Performance benchmarking across platforms

**Core Files Status**:
- ✅ `.env.example`: Complete configuration template
- ✅ `.env.local`: Local testing with Wise + local notary
- ✅ `src/lib.rs`: Enhanced data limits and utility functions
- ✅ `attestation/prove.rs`: Dual-phase MPC-TLS implementation
- ✅ `attestation/present.rs`: Dual-request selective disclosure
- ✅ `attestation/verify.rs`: Enhanced dual-phase verification
- ✅ `CLAUDE.md`: Updated with manual credential extraction focus
- ✅ `README.md`: Updated with manual credential extraction instructions
- ✅ `doc/PRD.md`: Updated with production binary requirements

**Next Steps for Binary Development**:
1. Create `src/main.rs` with comprehensive CLI interface using `clap`
2. Add JSON-based configuration parsing and validation
3. Implement structured error handling with clear error codes
4. Add cross-platform file I/O optimizations
5. Create build scripts for multi-target compilation
6. Add performance monitoring and logging capabilities