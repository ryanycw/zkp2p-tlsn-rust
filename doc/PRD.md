# Product Requirements Document (PRD)
## Wise.com TLSNotary Transaction Attestation for ZKP2P

**Version:** 1.0  
**Date:** January 2025  
**Author:** ZKP2P Development Team  

---

## Executive Summary

This PRD outlines the implementation of ZKP2P payment verification using TLSNotary for Wise.com, leveraging the Ethereum Foundation's Privacy and Scaling Explorations (PSE) protocol. **The primary deliverable is a production-ready binary prover that generates cryptographic proofs of fiat payment completion. This binary will be distributed to the [ZKP2P React Native SDK](https://github.com/zkp2p/zkp2p-react-native-sdk) repository for mobile integration.** The system uses Multi-Party Computation (MPC) TLS with direct transaction detail attestation to enable buyers to prove payment completion to sellers while maintaining complete privacy of sensitive financial data.

## 1. Product Overview

### 1.1 Problem Statement
- **Cryptographic Payment Verification**: ZKP2P buyers need to prove fiat payment completion to sellers without revealing sensitive financial data
- **Binary Distribution Requirements**: Mobile and desktop applications need a reliable, standalone binary for payment verification
- **Cross-Platform Compatibility**: Payment verification must work across multiple platforms (iOS, Android, macOS, Linux, Windows)
- **Developer Integration**: External systems need a simple CLI interface to integrate cryptographic proving capabilities
- **Performance Optimization**: The binary must be optimized for resource-constrained environments
- **Traditional Limitations**: Existing escrow systems require trusted intermediaries or manual verification processes
- **Privacy Requirements**: Users cannot safely share banking credentials or account details for payment verification

### 1.2 Solution
A ZKP2P payment verification system leveraging **TLSNotary MPC-TLS protocol** that:
- **Direct Attestation**: Directly attests specific payment details for ZKP2P verification requirements
- Establishes MPC-TLS connection where Notary participates in verification without seeing payment details
- Provides selective disclosure of payment data proving only essential verification facts
- Maintains complete privacy of session credentials, account information, and personal data
- Generates portable, cryptographic payment proofs signed by trusted Notary servers
- Delivers verifiable payment completion evidence for ZKP2P smart contracts enabling automated escrow release

### 1.3 Success Metrics
- Successfully attest Wise transaction data with 99.9% reliability using direct attestation
- Generate proofs in under 30 seconds for single-phase operations
- Support all major Wise transaction types
- Zero exposure of user authentication credentials in proofs
- Maintain complete privacy of sensitive financial data

## 2. User Stories

### 2.1 Primary Users
- **DeFi Users**: Want to prove fiat payments for crypto purchases across multiple platforms
- **ZKP2P Participants**: Need payment verification for peer-to-peer settlements
- **Application Developers**: Building applications requiring payment verification via binary integration
- **System Integrators**: Integrating ZKP2P payment proving into existing platforms and services

### 2.2 Core User Flows

#### User Story 1: Payment Proof Generation
**As a** ZKP2P user  
**I want to** create a cryptographic proof of my Wise payment  
**So that** I can automatically receive crypto without manual verification  

**Acceptance Criteria:**
- User provides Wise session credentials via environment variables
- System generates attestation of specific transaction
- Proof includes payment amount, currency, date, and completion status
- Authentication headers are hidden from the final proof

#### User Story 2: Payment Verification
**As a** ZKP2P verifier  
**I want to** verify payment proofs without trusting the prover  
**So that** I can safely release crypto assets  

**Acceptance Criteria:**
- Verify cryptographic integrity of payment data
- Confirm transaction details match expected values
- Validate proof was generated from legitimate Wise.com web interface
- No access to user's private authentication data

#### User Story 3: Selective Data Disclosure
**As a** privacy-conscious user  
**I want to** control what payment information is revealed  
**So that** I maintain financial privacy while proving payment  

**Acceptance Criteria:**
- Reveal only necessary transaction fields
- Hide sensitive recipient details when not required
- Maintain proof integrity with partial data disclosure

#### User Story 4: Binary Integration
**As an** application developer  
**I want to** integrate ZKP2P payment proving into my application via a simple CLI interface  
**So that** I can offer users cryptographic payment verification without implementing complex TLS protocols  

**Acceptance Criteria:**
- Binary provides comprehensive CLI interface with clear documentation
- JSON-based configuration and result formats for easy system integration
- Cross-platform binary compilation for major platforms
- Comprehensive error handling with clear error codes and messages
- File-based I/O for seamless integration with various systems

#### User Story 5: Performance Optimization
**As a** system administrator  
**I want to** run payment verification efficiently across different environments  
**So that** I can deploy ZKP2P proving in resource-constrained or high-throughput scenarios  

**Acceptance Criteria:**
- Payment proof generation completes within reasonable time limits
- Memory usage is optimized for various deployment environments
- Network usage is efficient and handles interruptions gracefully
- CPU usage is optimized for battery conservation on mobile deployments
- Binary size is minimized for distribution efficiency

## 3. Technical Requirements

### 3.1 Functional Requirements

#### 3.1.1 Core Attestation Features
- **F-001**: Generate TLSNotary attestation for Wise.com transaction verification
- **F-002**: Support transaction details endpoint:
  - `/gateway/v3/profiles/{PROFILE_ID}/transfers/{TRANSACTION_ID}` (payment details)
- **F-003**: Handle Cookie and X-Access-Token web session authentication
- **F-004**: Create selective disclosure presentations from transaction attestations
- **F-005**: Verify transaction attestations cryptographically

#### 3.1.2 Data Fields Support
**Payment Details Attestation:**
- **F-006**: Attest transaction amount (`primaryAmount`)
- **F-007**: Attest transaction currency (`currency`)
- **F-008**: Attest transaction ID (`resource.id`)
- **F-009**: Attest transaction date (`visibleOn`)
- **F-010**: Attest transaction status (`status`)
- **F-011**: Attest recipient information (`title`)

#### 3.1.3 Security Requirements
- **F-012**: Hide web session credentials in presentations
- **F-013**: Use production TLS verification for Wise.com
- **F-014**: Support configurable notary server connections
- **F-015**: Validate certificate chains for production endpoints
- **F-016**: Ensure MPC-TLS session integrity

#### 3.1.4 Binary CLI Requirements
- **F-017**: Create production binary with comprehensive CLI interface using `clap`
- **F-018**: Support JSON-based configuration input and structured result output
- **F-019**: Implement clear error codes and messages for external integration
- **F-020**: Handle various file system scenarios and permissions
- **F-021**: Support cross-platform compilation for iOS, Android, and desktop targets
- **F-022**: Optimize binary size for efficient distribution
- **F-023**: Provide detailed logging and debugging capabilities

#### 3.1.5 Performance and Integration Requirements
- **F-024**: Optimize memory usage for resource-constrained environments
- **F-025**: Handle network interruptions and reconnection gracefully
- **F-026**: Support efficient file-based I/O operations
- **F-027**: Implement progress reporting through standard output
- **F-028**: Provide comprehensive documentation and examples

### 3.2 Non-Functional Requirements

#### 3.2.1 Performance
- **NF-001**: Generate transaction attestations within reasonable time limits for target environment
- **NF-002**: Support up to 64KB response data from Wise transaction endpoint
- **NF-003**: Handle concurrent transaction attestation requests
- **NF-004**: Optimize MPC-TLS session for transaction details
- **NF-005**: Memory usage optimized for deployment environment constraints
- **NF-006**: Binary size optimized for efficient distribution across platforms
- **NF-007**: CPU usage optimized for battery conservation in mobile deployments

#### 3.2.2 Reliability
- **NF-004**: 99.9% uptime for attestation generation
- **NF-005**: Graceful handling of network timeouts
- **NF-006**: Retry logic for failed API connections

#### 3.2.3 Security
- **NF-007**: Zero-knowledge proof of payment data
- **NF-008**: No storage of user credentials
- **NF-009**: Cryptographic integrity of all attestations
- **NF-010**: Protection against man-in-the-middle attacks

#### 3.2.4 Compatibility
- **NF-011**: Compatible with ZKP2P provider specification 
- **NF-012**: Support for multiple Wise account currencies
- **NF-013**: Backward compatibility with existing test fixtures
- **NF-014**: Cross-platform binary compatibility across target platforms
- **NF-015**: JSON output format compatible with various integration scenarios

## 4. Technical Architecture

### 4.1 TLSNotary System Components

#### 4.1.1 Prover Component (Modular Architecture)

**Core Modules (`src/`):**
- **`attestation/`**: Attestation operations - transcript analysis, commitment creation, notarization
- **`http/`**: HTTP request building with header management and sensitive data redaction
- **`notary/`**: Notary server configuration, client setup, and notarization requests
- **`providers/wise.rs`**: Wise-specific dual-phase logic and credential management
- **`tls/`**: MPC-TLS session management, crypto provider setup, prover configuration

**Orchestration (`attestation/prove.rs`):**
- **Role**: Coordinates modules to execute TLS 1.2 connection with Wise.com
- **Transaction Attestation**: Directly attests specific transaction details for payment verification
- **MPC-TLS**: Manages secret-shared TLS session keys with Notary via module coordination
- **Transparency**: Wise.com sees only standard browser traffic (Notary is invisible to server)
- **Authentication**: Delegates credential handling to providers module
- **Output**: Generates cryptographic attestations signed by Notary server

#### 4.1.2 Notary Server (External)
- **Role**: Trusted third-party that participates in MPC-TLS without seeing data content
- **Verification**: Cryptographically signs commitments to data authenticity and server identity
- **Privacy**: Never learns plaintext data or server identity during MPC operations
- **Portability**: Creates reusable, portable attestations for later verification
- **Trust Model**: Verifiers can require multiple Notary signatures to prevent collusion

#### 4.1.3 Presenter Component (`attestation/present.rs`)
- **Selective Disclosure**: Creates privacy-preserving presentations from transaction attestations
- **Payment Disclosure**: Reveals essential payment verification fields
- **Zero-Knowledge**: Can prove properties of data without revealing exact values
- **Field Control**: Reveals only specified Wise transaction fields (amount, currency, date, etc.)
- **Privacy Protection**: Hides sensitive authentication headers from all parties
- **ZKP2P Integration**: Formats transaction data according to ZKP2P provider specifications

#### 4.1.4 Verifier Component (`attestation/verify.rs`)
- **Trustless Verification**: Validates cryptographic proofs without trusting the Prover
- **Payment Validation**: Verifies specific payment details match ZKP2P requirements
- **Attestation Validation**: Confirms Notary's cryptographic signature on transaction data commitments
- **Presentation Integrity**: Verifies selective disclosure hasn't compromised proof validity
- **Certificate Validation**: Uses production TLS certificate chains for Wise.com verification

### 4.2 Binary Architecture Overview

#### 4.2.1 System Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                    External Applications                            │
│  ┌─────────────────┬─────────────────┬─────────────────────────────┐│
│  │  React Native   │   Desktop App   │      Server Application    ││
│  │   - Mobile UI   │   - GUI Tools   │      - API Service         ││
│  │   - Integration │   - Dev Tools   │      - Batch Processing    ││
│  └─────────────────┴─────────────────┴─────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────┤
│                    CLI Interface Integration                        │
│   Process execution with JSON config input/output                  │
│   Standard error codes for integration handling                    │
│   File-based I/O for cross-platform compatibility                 │
├─────────────────────────────────────────────────────────────────────┤
│                    zkp2p-tlsn-rust Binary                          │
│  ┌─────────────────────────────────────────────────────────────────┐│
│  │                      CLI Interface                             ││
│  │   zkp2p-tlsn-prover prove --config config.json               ││
│  │   zkp2p-tlsn-prover present --attestation att.tlsn           ││
│  │   zkp2p-tlsn-prover verify --presentation pres.tlsn          ││
│  ├─────────────────────────────────────────────────────────────────┤│
│  │                 Modular Architecture                           ││
│  │   • http/: Request building and header management             ││
│  │   • notary/: Notary server configuration and connection       ││
│  │   • providers/: Provider-specific implementations (Wise)      ││
│  │   • tls/: MPC-TLS session and crypto management              ││
│  │   • attestation/: Commitment and notarization operations      ││
│  ├─────────────────────────────────────────────────────────────────┤│
│  │                   Dual-Phase TLS Prover                       ││
│  │   Phase 1: Transaction ownership verification                 ││
│  │   Phase 2: Payment details attestation                       ││
│  │   MPC-TLS with Notary server coordination                     ││
│  │   JSON-based configuration and output handling               ││
│  └─────────────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────────────┤
│                        External Dependencies                       │
│  ┌─────────────────┬─────────────────┬─────────────────────────────┐│
│  │   Notary Server │    Wise.com     │      File System           ││
│  │  - MPC-TLS      │  - Payment API  │    - Config Files          ││
│  │  - Attestation  │  - Auth Session │    - Output Files          ││
│  │  - Verification │  - Transaction  │    - Temp Files             ││
│  └─────────────────┴─────────────────┴─────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────┘
```

#### 4.2.2 Binary CLI Specification

**Configuration JSON Schema:**
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
    "directory": "/path/to/output/",
    "format": "json",
    "attestation_file": "attestation.tlsn",
    "secrets_file": "secrets.tlsn",
    "presentation_file": "presentation.tlsn"
  }
}
```

**Output JSON Schema:**
```json
{
  "status": "success|error",
  "operation": "prove|present|verify",
  "files": {
    "attestation": "/path/to/attestation.tlsn",
    "secrets": "/path/to/secrets.tlsn",
    "presentation": "/path/to/presentation.tlsn"
  },
  "metadata": {
    "transaction_id": "987654321",
    "proof_generated_at": "2025-01-24T10:30:00Z",
    "duration_ms": 45000
  },
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable error message",
    "details": {}
  }
}
```

### 4.3 Integration Architecture Alternatives

This section outlines different approaches for integrating the ZKP2P TLSNotary binary with external applications, particularly React Native mobile apps and other systems requiring payment verification capabilities.

#### 4.3.1 Alternative 1: CLI Binary Interface (Chosen)

**Overview**: Standalone executable binary with comprehensive command-line interface that external applications invoke as a subprocess.

**Architecture**:
```
External App → Process Execution → zkp2p-tlsn-prover CLI → JSON Output → External App
```

**Implementation Details**:
- `clap`-based CLI with structured commands (`prove`, `present`, `verify`)
- JSON-based configuration input via files or environment variables
- Structured JSON output with success/error status and metadata
- Standard error codes for reliable integration handling
- File-based I/O for cross-platform compatibility

**Advantages**:
- **Simplicity**: No complex FFI bindings or integration code required
- **Platform Independence**: Single binary works across iOS, Android, desktop
- **Language Agnostic**: Any language can execute processes and parse JSON
- **Debugging**: Easy to test and debug CLI operations independently
- **Distribution**: Simple binary distribution without library dependencies
- **Security**: Process isolation provides natural security boundaries

**Disadvantages**:
- **Performance**: Process execution overhead (~100ms startup time)
- **Resource Usage**: Each invocation creates new process and memory space
- **File I/O Dependency**: Requires filesystem access for configuration/output

**React Native Integration**:
```javascript
import { exec } from 'react-native-process';

const config = {
  wise: { profile_id: "123", transaction_id: "456", /* ... */ },
  notary: { host: "notary.pse.dev", port: 7047 }
};

const result = await exec('zkp2p-tlsn-prover', [
  'prove', 
  '--config', 
  JSON.stringify(config)
]);
const proofData = JSON.parse(result.stdout);
```

#### 4.3.2 Alternative 2: FFI (Foreign Function Interface)

**Overview**: Expose Rust functions as C-compatible library interface for direct function calls from external applications.

**Architecture**:
```
External App → FFI Bridge → Rust Library Functions → Direct Return Values
```

**Implementation Details**:
- `#[no_mangle] extern "C"` functions for key operations
- C header files defining function signatures and data structures
- Memory management across language boundaries
- Error handling via return codes and out parameters
- Cross-compilation to static/dynamic libraries per platform

**Advantages**:
- **Performance**: Direct function calls eliminate process overhead
- **Memory Efficiency**: Shared memory space, no serialization overhead
- **Real-time Integration**: Immediate function returns without process management
- **Resource Sharing**: Can maintain persistent connections and state

**Disadvantages**:
- **Complexity**: Requires platform-specific FFI bridge code for each language
- **Memory Management**: Complex ownership and lifetime management across boundaries
- **Platform Compilation**: Separate library builds required for each target platform
- **Debugging Difficulty**: Cross-language debugging is significantly more complex
- **Error Propagation**: Complex error handling across language boundaries
- **Version Management**: Library versioning and compatibility challenges

**React Native Integration**:
```javascript
import { NativeModules } from 'react-native';
const { ZKP2PTLSNProver } = NativeModules;

const result = await ZKP2PTLSNProver.prove({
  wiseProfileId: "123",
  wiseTransactionId: "456",
  // ... other config
});
```

#### 4.3.3 Alternative 3: WebAssembly (WASM)

**Overview**: Compile Rust code to WebAssembly for execution in JavaScript environments with near-native performance.

**Architecture**:
```
External App → WASM Runtime → Compiled Rust Functions → Direct Memory Access
```

**Implementation Details**:
- `wasm-pack` compilation to generate WASM modules and TypeScript bindings
- JavaScript/TypeScript wrapper functions for type safety
- Memory management via WASM linear memory model
- Async/await support for network operations
- bundler integration for web and React Native environments

**Advantages**:
- **Performance**: Near-native execution speed within JavaScript runtime
- **Cross-Platform**: Single WASM build works across web and mobile platforms
- **Integration**: Natural JavaScript integration with TypeScript support
- **Portability**: No platform-specific compilation required
- **Sandbox Security**: WASM provides memory-safe execution environment

**Disadvantages**:
- **Runtime Requirements**: Requires WASM runtime support (not available in all environments)
- **Bundle Size**: WASM modules can be large, impacting app distribution size
- **Network Limitations**: WASM network access requires host environment permissions
- **Debugging**: Limited debugging capabilities compared to native code
- **TLS Complexity**: TLS operations may require host environment support

**React Native Integration**:
```javascript
import * as zkp2pWasm from 'zkp2p-tlsn-wasm';

await zkp2pWasm.default(); // Initialize WASM module
const result = await zkp2pWasm.prove({
  wise_profile_id: "123",
  wise_transaction_id: "456",
  // ... other config
});
```

#### 4.3.4 Architecture Decision Rationale

**Why CLI Binary Interface was chosen**:

1. **Simplicity First**: The CLI approach provides the simplest integration path for the zkp2p-react-native-sdk, allowing developers to focus on mobile UI/UX rather than complex FFI bindings.

2. **Cross-Platform Compatibility**: A single binary approach eliminates the need for platform-specific compilation matrices and FFI bridge code, reducing maintenance overhead.

3. **Development Velocity**: CLI integration can be implemented and tested immediately without waiting for complex FFI infrastructure, accelerating time-to-market.

4. **Security Benefits**: Process isolation provides natural security boundaries and prevents potential memory corruption issues that could arise from FFI implementations.

5. **Future Flexibility**: The CLI interface can serve as a foundation for future FFI or WASM implementations while providing immediate functionality.

6. **Distribution Simplicity**: Binary distribution to the zkp2p-react-native-sdk repository is straightforward without requiring platform-specific library builds.

The CLI approach represents the optimal balance of implementation simplicity, integration reliability, and cross-platform compatibility for the current requirements, while maintaining the flexibility to evolve toward more performance-optimized approaches as the ecosystem matures.

### 4.4 Dual-Phase MPC-TLS Protocol Data Flow

```
Phase 1: MPC-TLS Connection Setup
1. User provides credentials → Prover environment variables
2. Prover initiates TLS 1.2 handshake with Wise.com
3. Prover ↔ Notary: Secret-share TLS session keys via MPC
4. Wise.com ← Prover: Sees standard TLS connection (Notary transparent)

Phase 2A: Transaction List Ownership Verification
5. Prover → Wise.com: GET /all-transactions?direction=OUTGOING
6. Wise.com → Prover: Transaction list JSON response
7. Prover: Verifies target transaction exists in user's list
8. Notary: Participates in MPC without seeing plaintext or server identity

Phase 2B: Transaction Details Attestation
9. Prover → Wise.com: GET /gateway/v3/profiles/{PROFILE_ID}/transfers/{TRANSACTION_ID}
10. Wise.com → Prover: Specific transaction JSON response
11. Notary: Continues MPC participation for payment details

Phase 3: Cryptographic Attestation
12. Notary: Creates cryptographic commitments to dual-phase data + server identity
13. Notary → Prover: Signed dual-phase attestation (portable & reusable)

Phase 4: Selective Disclosure & Verification
14. Prover → Presenter: Creates selective disclosure from dual-phase attestation
15. Presenter: Reveals ownership proof + essential payment fields, hides sensitive data
16. Presentation → Verifier: Validates dual-phase proof using Notary's cryptographic signature
17. Verifier → ZKP2P Application: Enhanced trustless verification result for settlement
```

### 4.5 Security Model

#### 4.5.1 Trust Assumptions
- **Notary Trust**: User trusts selected Notary server(s) for honest MPC participation
- **Protocol Trust**: All parties trust TLSNotary's MPC-TLS cryptographic protocols (PSE/Ethereum Foundation)
- **Zero Prover Trust**: Verifiers require no trust in Prover - cryptographic guarantees only
- **Multi-Notary Option**: Verifiers can require multiple Notary signatures to prevent single-point-of-trust

#### 4.5.2 MPC-TLS Threat Model
- **Malicious Prover**: 
  - Cannot forge transaction data due to MPC-TLS cryptographic guarantees
  - Cannot modify server responses without detection
  - Cannot create false attestations without Notary cooperation
- **Malicious Notary**: 
  - Cannot see plaintext data or server identity during MPC
  - Cannot forge data without Prover's participation
  - Can be mitigated by requiring multiple Notary signatures
- **Network Attacks**: 
  - MPC-TLS provides protection against man-in-the-middle attacks
  - Standard TLS encryption protects against eavesdropping
  - Notary participation doesn't compromise connection security
- **Credential Protection**: 
  - Authentication data (Cookie, X-Access-Token) never exposed in final proofs
  - Selective disclosure prevents accidental credential revelation
  - Zero-knowledge proofs maintain privacy of sensitive data

## 5. Configuration & Environment

### 5.1 Environment Variables

#### 5.1.1 Required for Wise Attestation
```bash
WISE_PROFILE_ID="user_profile_id"           # Wise user profile
WISE_TRANSACTION_ID="transaction_id"        # Target transaction
WISE_COOKIE="session_cookie"                # Authentication cookie
WISE_ACCESS_TOKEN="api_access_token"        # API access token
```

#### 5.1.2 Optional Configuration
```bash
WISE_HOST="wise.com"                        # Wise API host
WISE_PORT="443"                             # HTTPS port
NOTARY_HOST="notary.pse.dev"               # Production notary
NOTARY_PORT="7047"                          # Notary port
NOTARY_TLS="true"                           # Enable notary TLS
```

### 5.2 Usage Commands

```bash
# Generate Wise transaction attestation
cargo run --release --example attestation_prove -- wise-transaction

# Create selective presentation
cargo run --release --example attestation_present -- wise-transaction

# Verify presentation
cargo run --release --example attestation_verify -- wise-transaction
```

## 6. Integration Requirements

### 6.1 ZKP2P Provider Compatibility
- Matches ZKP2P Wise provider specification from `providers/wise/transfer_wise.json`
- Supports JSONPath selectors: `$.primaryAmount`, `$.currency`, `$.resource.id`, `$.visibleOn`, `$.title`
- Compatible with existing ZKP2P verification infrastructure

### 6.2 Web Interface Requirements
- Wise web interface v3 gateway endpoint support
- Session-based authentication handling
- JSON response parsing and selective extraction

## 7. Deployment & Operations

### 7.1 Mobile Deployment Requirements

#### 7.1.1 Development Environment
- Rust 1.70+ with Cargo and cross-compilation targets
- React Native development environment (Node.js, npm/yarn)
- Android SDK and NDK for Android targets
- Xcode and iOS SDK for iOS targets
- Access to production notary servers
- TLSNotary v0.1.0-alpha.12 dependencies

#### 7.1.2 Cross-Platform Compilation Targets
```bash
# iOS Targets
rustup target add aarch64-apple-ios          # iOS devices
rustup target add x86_64-apple-ios           # iOS Simulator

# Android Targets
rustup target add aarch64-linux-android      # Android ARM64
rustup target add armv7-linux-androideabi    # Android ARM32
rustup target add i686-linux-android         # Android x86
rustup target add x86_64-linux-android       # Android x86_64
```

#### 7.1.3 Binary Distribution
- **CLI Binary**: Standalone executable for development testing
- **Static Library**: For React Native FFI integration (`libzkp2p_tlsn_rust.a`)
- **Dynamic Library**: For runtime linking (`libzkp2p_tlsn_rust.so/.dylib`)
- **Header Files**: C interface definitions for FFI bridge

#### 7.1.4 Mobile App Integration
- Binary embedded within React Native app bundle
- File I/O configured for mobile app sandbox directories
- Network permissions configured for TLS connections
- Background processing capabilities for long-running operations

### 7.2 Monitoring & Logging
- Attestation success/failure rates
- Response times for proof generation
- Network connectivity issues
- Authentication failures (without credential exposure)

### 7.3 Error Handling
- Invalid credentials: Clear error messages
- Network timeouts: Automatic retry with backoff
- Malformed API responses: Graceful degradation
- Notary server unavailability: Alternative server support

## 8. Testing Strategy

### 8.1 Unit Tests
- Transaction data parsing
- Selective disclosure logic
- Authentication header handling
- Error condition responses

### 8.2 Integration Tests
- End-to-end attestation flow
- Wise web interface compatibility  
- Notary server integration
- Production TLS verification
- React Native FFI integration testing
- Cross-platform compilation verification
- Mobile app sandbox file I/O testing

### 8.3 Mobile-Specific Testing
- Performance testing on actual iOS/Android devices
- Memory usage profiling during proving operations  
- Battery usage optimization verification
- Network interruption and reconnection handling
- Background operation testing
- App store compliance verification

### 8.4 Local Testing Setup

#### 8.4.1 Testing with Real Wise Server + Local Notary

**Prerequisites:**
1. Clone TLSNotary repository: `git clone https://github.com/tlsnotary/tlsn.git`
2. Active Wise account with recent transactions
3. Rust development environment with mobile targets

**Step-by-Step Local Testing:**

```bash
# Terminal 1 - Start Local Notary Server
cd /path/to/tlsn
cargo run --release --bin notary-server

# Expected: "INFO notary_server: Notary server listening on 127.0.0.1:7047"

# Terminal 2 - Configure Test Environment  
cd /path/to/zkp2p-tlsn-rust
cp .env.local .env

# Edit .env with real Wise credentials:
# 1. Login to wise.com in browser
# 2. Open Developer Tools (F12) → Network tab
# 3. Refresh page, click any wise.com request
# 4. Copy 'Cookie' and 'X-Access-Token' header values
# 5. Get Profile ID from account settings
# 6. Get Transaction ID from transaction history

# Terminal 2 - Run Dual-Phase Test
cargo run --release --example attestation_prove -- wise-transaction
cargo run --release --example attestation_present -- wise-transaction  
cargo run --release --example attestation_verify -- wise-transaction
```

**Environment Configuration (`.env`):**
```bash
# Real Wise credentials
WISE_PROFILE_ID=12345678
WISE_TRANSACTION_ID=987654321
WISE_COOKIE="session_id=abc123; csrf_token=xyz789"
WISE_ACCESS_TOKEN=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.example

# Local notary for testing
NOTARY_HOST=127.0.0.1
NOTARY_PORT=7047
NOTARY_TLS=false

# Production Wise server
WISE_HOST=wise.com
WISE_PORT=443
```

**Expected Test Output:**
```
🚀 Starting ZKP2P payment verification via TLSNotary...
📡 Connecting to Notary server: 127.0.0.1:7047
✅ Notary connection established
🔄 Executing dual-phase MPC-TLS requests:
   Phase 1: Verifying transaction ownership...
   ✅ Transaction ownership verified: [ID] found in user's list
   Phase 2: Attesting transaction details...  
   ✅ Transaction details retrieved successfully
🎉 ZKP2P dual-phase payment proof generated successfully!
```

#### 8.4.2 Mobile Integration Testing

**React Native SDK Integration Test:**
```bash
# Build for mobile targets
cargo build --release --target aarch64-apple-ios
cargo build --release --target aarch64-linux-android

# Test FFI interface
cd /path/to/zkp2p-react-native-sdk
npm install
npm run test:integration

# Run on actual devices  
npx react-native run-ios --device
npx react-native run-android --device
```

### 8.3 Security Tests
- Credential exposure prevention
- Man-in-the-middle attack resistance
- Cryptographic proof integrity
- Selective disclosure correctness

## 9. Risk Assessment

### 9.1 Technical Risks
- **Wise Web Interface Changes**: Medium risk - Interface evolution may break integration
- **TLS Certificate Issues**: Low risk - Standard certificate validation
- **Notary Server Availability**: Medium risk - Requires backup servers

### 9.2 Security Risks
- **Credential Exposure**: Low risk - Credentials hidden in presentations
- **Proof Forgery**: Very low risk - Cryptographic protection
- **Privacy Leakage**: Low risk - Selective disclosure controls

### 9.3 Mitigation Strategies
- Regular web interface compatibility testing
- Multiple notary server support
- Comprehensive error handling
- Security audit of disclosure logic

## 10. Future Enhancements

### 10.1 Phase 2 Features
- Support for multiple transaction attestation
- Batch proof generation
- Additional Wise web endpoints (balance, profile)
- Mobile SDK integration

### 10.2 Advanced Features
- Zero-knowledge balance proofs (prove balance > X without revealing amount)
- Multi-currency conversion attestations
- Recurring payment verification
- Cross-border compliance proofs

## 11. Success Criteria

### 11.1 Launch Criteria

**Phase 1: Core Implementation (✅ Complete)**
- ✅ Successful dual-phase attestation of Wise transaction data
- ✅ Selective disclosure of payment information with privacy preservation
- ✅ Integration with ZKP2P provider specification  
- ✅ Production TLS verification support
- ✅ Comprehensive documentation and examples

**Phase 2: Binary Development (🔄 In Progress)**
- 🔄 Production binary with comprehensive CLI interface
- 🔄 JSON-based configuration and output handling
- 🔄 Cross-platform compilation for iOS, Android, and desktop
- 🔄 Performance optimization for resource-constrained environments
- 🔄 Comprehensive error handling and logging

**Phase 3: Binary Distribution (📋 Planned)**
- 📋 Cross-platform binary builds and testing
- 📋 Binary distribution to zkp2p-react-native-sdk repository
- 📋 Integration testing with external applications
- 📋 Performance benchmarking across platforms

### 11.2 Post-Launch Metrics

**Core Performance:**
- Dual-phase attestation success rate > 99%
- Average proof generation time < 60s on mobile devices
- Memory usage < 100MB during operations
- Zero credential exposure incidents

**Binary Integration:**
- Cross-platform compilation success rate > 95%
- Binary size optimized for distribution efficiency
- Performance optimization for resource-constrained environments
- External application integration success rate > 90%

**Developer Experience:**
- CLI integration complexity minimal for external applications
- Documentation completeness score > 90%
- Developer feedback and issue resolution time < 48hrs
- JSON-based configuration and output ease of use

---

**Document Status**: ✅ Complete - Updated for Binary Distribution Focus  
**Implementation Status**: 
- ✅ Phase 1: Dual-Phase TLSNotary Core Complete
- 🔄 Phase 2: Binary Development In Progress  
- 📋 Phase 3: Binary Distribution Planned

**Review Required**: Security audit recommended before mainnet deployment (enhanced for cross-platform binary distribution)