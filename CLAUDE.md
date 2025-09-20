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

# FFI Development (see Makefile targets)
make help                    # Show all available targets
make build-rust              # Build with C bindings generation
make build-cross-platform    # Cross-platform compilation
make test                    # Build and test FFI interface
make check-deps              # Verify required dependencies
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

## FFI (Foreign Function Interface)

The project includes comprehensive FFI support for cross-platform integration:

### FFI Components

- **Auto-generated C headers** (`include/zkp2p_ffi.h`) via cbindgen in `build.rs`
- **Cross-platform compilation** via `build-cross-platform.sh` script
- **Makefile automation** for building, testing, and dependency checking
- **C test suite** (`tests/test_ffi.c`) for validation

### FFI Functions

```c
// Core FFI interface
int32_t zkp2p_init(void);
void zkp2p_cleanup(void);
int32_t zkp2p_prove(int32_t mode, int32_t provider, const char *transaction_id,
                    const char *profile_id, const char *cookie, const char *access_token);
int32_t zkp2p_verify(int32_t provider);
const char *zkp2p_get_last_error(void);
void zkp2p_free_error_string(char *ptr);
```

### Target Platforms

The cross-platform build script supports:

- **iOS** (macOS host only):
  - aarch64-apple-ios (ARM64 devices)
  - x86_64-apple-ios (Intel simulator)
  - aarch64-apple-ios-sim (Apple Silicon simulator)
- **Android** (via cargo-ndk):
  - aarch64-linux-android (ARM64)
  - armv7-linux-androideabi (ARMv7)
  - i686-linux-android (x86)
  - x86_64-linux-android (x86_64)

Note: Desktop platforms (Linux, Windows, macOS) use standard `cargo build --release` for native development.

### FFI Build Process

1. **C Header Generation**: `build.rs` automatically generates `include/zkp2p_ffi.h` using cbindgen
2. **Cross-Platform Build**: `build-cross-platform.sh` compiles for all supported targets
3. **Library Packaging**: Organized output in `libs/` directory structure
4. **Testing**: Automated C FFI tests ensure interface compatibility

## Current Status

- ✅ Working CLI binaries
- ✅ Wise.com integration
- ✅ PayPal provider structure (implementation pending)
- ✅ Local and production notary support
- ✅ Comprehensive error handling
- ✅ Modular architecture
- ✅ Cross-platform FFI support
- ✅ Auto-generated C bindings
- ✅ Makefile build automation
- ✅ C test suite for FFI validation