use clap::{Parser, ValueEnum};
use std::fmt;

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum Mode {
    Prove,
    Present,
    ProveToPresent,
}

#[derive(Debug, Clone, ValueEnum, PartialEq)]
pub enum Provider {
    Wise,
    PayPal,
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Provider::Wise => write!(f, "wise"),
            Provider::PayPal => write!(f, "paypal"),
        }
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "ZKP2P TLSNotary Prover - Proving and Presenting")]
pub struct ProveArgs {
    /// Operation mode
    #[clap(long, value_enum)]
    pub mode: Mode,
    /// API endpoint URL
    #[clap(
        long,
        required_if_eq("mode", "prove"),
        required_if_eq("mode", "prove_to_present")
    )]
    pub url: String,
    /// Transaction ID
    #[clap(
        long,
        required_if_eq("mode", "prove"),
        required_if_eq("mode", "prove_to_present")
    )]
    pub transaction_id: String,
    /// Session cookie
    #[clap(
        long,
        required_if_eq("mode", "prove"),
        required_if_eq("mode", "prove_to_present")
    )]
    pub cookie: Option<String>,
    /// Access token
    #[clap(
        long,
        required_if_eq("mode", "prove"),
        required_if_eq("mode", "prove_to_present")
    )]
    pub access_token: Option<String>,
}

#[derive(Parser, Debug)]
#[command(version, about = "ZKP2P TLSNotary Verifier - Verifying")]
pub struct VerifyArgs {
    /// API endpoint URL
    #[clap(long)]
    pub url: String,
    /// Transaction ID
    #[clap(long)]
    pub transaction_id: String,
}
