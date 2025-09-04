# zkp2p-tlsn-rust

Rust implementation of TLSNotary Attestation for ZKP2P - Single-phase transaction detail verification

## Configuration

### Environment Variables

Copy `.env.local` to `.env` and configure the following:

#### Wise Server Configuration

- `WISE_HOST`: Wise web interface hostname (default: wise.com)
- `WISE_PORT`: Wise HTTPS port (default: 443)

#### Notary Server Configuration

- `NOTARY_HOST`: Notary server hostname
  - Production: `notary.pse.dev`
  - Local development: `127.0.0.1`
- `NOTARY_PORT`: Notary server port (default: 7047)
- `NOTARY_TLS`: Enable TLS for notary connection
  - Production: `true`
  - Local development: `false`

## Usage Instructions

### Manual Credential Extraction

Due to Wise's modern security measures (Cloudflare Turnstile), credentials must be extracted manually from your browser:

1. **Login to Wise and extract credentials:**

   - Open your browser and login to [wise.com](https://wise.com)
   - Open Developer Tools (F12) and go to the **Network** tab
   - Navigate to any page that requires authentication (like your transaction history)
   - Look for any network request and click on it
   - In the **Request Headers** section, copy:
     - `Cookie` header value
     - `X-Access-Token` header value (if present)
   - Find your profile ID from your account settings or API responses

2. **Find your transaction ID:**

   - Go to wise.com/all-transactions?direction=OUTGOING
   - Find the payment you want to prove
   - Copy the transaction ID from the transaction details

3. **Generate payment proof:**

   ```bash
   cargo run --release --example attestation_prove -- wise-transaction \
     --wise-profile-id "your_profile_id" \
     --wise-transaction-id "your_transaction_id" \
     --wise-cookie "your_cookie_header" \
     --wise-access-token "your_access_token"
   ```

4. **Create selective presentation:**

   ```bash
   cargo run --release --example attestation_present -- wise-transaction
   ```

5. **Verify the proof:**

   ```bash
   cargo run --release --example attestation_verify -- wise-transaction
   ```

## Testing Setup

### Local Testing with Real Wise Server + Local Notary

This setup allows you to test against the actual Wise production server while using a local notary server for development. The system generates cryptographic proofs of payment completion through direct transaction detail attestation.

#### Prerequisites

1. Clone the TLSNotary repository:

   ```bash
   git clone https://github.com/tlsnotary/tlsn.git
   cd tlsn
   ```

2. Have an active Wise account with recent transactions

#### Setup Steps

1. **Terminal 1 - Start Local Notary Server:**

   ```bash
   cd tlsn
   cargo run --release --bin notary-server
   ```

   Expected output: "Notary server listening on 127.0.0.1:7047"

2. **Terminal 2 - Configure and Run:**

   ```bash
   # Copy local testing configuration
   cp .env.local .env

   # Extract credentials manually from browser (see Usage Instructions above)
   # Then run with your extracted credentials:
   cargo run --release --example attestation_prove -- wise-transaction \
     --wise-profile-id "your_profile_id" \
     --wise-transaction-id "your_transaction_id" \
     --wise-cookie "your_cookie_header" \
     --wise-access-token "your_access_token"
   ```

## Security Notes

- **Never commit `.env` files to version control** - they may contain sensitive credentials
- Handle extracted credentials securely - they provide access to your Wise account
- MPC-TLS ensures the notary server doesn't see plaintext data during transaction detail attestation
- Credentials are only used for the duration of the proving session
- The system directly attests payment details without exposing sensitive authentication data
- For production use, always use the official PSE notary server (`notary.pse.dev`)
- Wise uses modern security measures (Cloudflare Turnstile) that prevent automated login attempts

## Prerequisites

- Ruby & Pod for ZKP2P React Native
