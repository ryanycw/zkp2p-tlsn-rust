# zkp2p-tlsn-rust

Rust implementation of TLSNotary Attestation for ZKP2P

## Steps

1. Clone the repository
   https://github.com/tlsnotary/tlsn.git

2. Run the test Server + Notary Server

```
RUST_LOG=info PORT=4000 cargo run --bin tlsn-server-fixture
```

```
cargo run --release --bin notary-server
```

3. Run the prover

```
SERVER_PORT=4000 cargo run --release --example attestation_prove
```

## Prerequisites

- Ruby & Pod for ZKP2P React Native
