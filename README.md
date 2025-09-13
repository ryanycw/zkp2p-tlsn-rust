# zkp2p-tlsn-rust

ZKP2P payment verification using TLSNotary - proves Wise.com transaction completion cryptographically.

## What it does

Generates cryptographic proofs of fiat payments without revealing sensitive credentials or banking details. Uses TLSNotary's MPC-TLS protocol to prove payment completion for ZKP2P settlements.

## Quick start

### 1. Setup

```bash
# Clone and build
git clone https://github.com/yourusername/zkp2p-tlsn-rust
cd zkp2p-tlsn-rust
cargo build --release

# Copy configuration template
cp .env.local .env
```

### 2. Get your credentials

**Manual extraction required** - Wise uses modern security that prevents automation:

1. Login to [wise.com](https://wise.com) in your browser
2. Open Developer Tools (F12) → Network tab
3. Navigate to any authenticated page
4. Click any wise.com request and copy from Request Headers:
   - `Cookie` header value
   - `X-Access-Token` header value
5. Get your Profile ID from account settings
6. Find Transaction ID at wise.com/all-transactions

### 3. Generate proof

```bash
# Create cryptographic proof
cargo run --release --bin zkp2p-prove \
  --mode prove-to-present \
  --provider wise \
  --profile-id "12345678" \
  --transaction-id "987654321" \
  --cookie "session_id=abc123..." \
  --access-token "eyJhbGciOiJIUzI1NiI..."
```

### 4. Create presentation

```bash
# Create selective disclosure (reveals only essential payment fields)
cargo run --release --bin zkp2p-prove \
  --mode present \
  --provider wise
```

### 5. Verify

```bash
# Verify the proof (typically done by smart contracts)
cargo run --release --bin zkp2p-verify
```

## Configuration

### Environment variables (.env)

```bash
# Notary server (production)
NOTARY_HOST=notary.pse.dev
NOTARY_PORT=7047
NOTARY_TLS=true

# Wise server
WISE_HOST=wise.com
WISE_PORT=443
```

### Local testing setup

For development with local notary server:

```bash
# Terminal 1 - Start local notary
cd /path/to/tlsn
cargo run --release --bin notary-server

# Terminal 2 - Use local notary
cp .env.local .env
# Then run prove command as above
```

## What gets proven

The proof reveals only essential payment fields:

- Payment amount and currency
- Transaction ID
- Payment completion status
- Recipient identifier
- Payment timestamp

**Privacy**: Session credentials, account details, and personal information stay completely private.

## CLI Options

```bash
zkp2p-prove --help           # See all options
zkp2p-verify --help          # Verification options
```

### Modes

- `prove` - Generate attestation
- `present` - Create selective disclosure
- `prove-to-present` - Do both in one step

### Providers

- `wise` - Wise.com payments
- `paypal` - PayPal payments

## Requirements

- Rust 1.70+
- Active Wise account with transaction history
- Network access to notary server

## Security

- Zero-knowledge proof generation
- No credential storage
- MPC-TLS ensures notary never sees your data
- Cryptographic guarantees via Ethereum Foundation's TLSNotary protocol

## Files generated

- `wise-attestation.tlsn` - Cryptographic attestation
- `wise-secrets.tlsn` - Secret data for presentations
- `wise-presentation.tlsn` - Selective disclosure proof
