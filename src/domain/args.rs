use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, ValueEnum)]
pub enum Mode {
    Prove,
    Present,
    ProveToPresent,
}

#[derive(Parser, Debug)]
#[command(version, about = "ZKP2P TLSNotary Prover - Proving and Presenting")]
pub struct ProveArgs {
    /// Operation mode
    #[clap(short, long, value_enum)]
    pub mode: Mode,
    #[clap(long)]
    pub provider: String,
    /// Profile ID
    #[clap(long)]
    pub profile_id: Option<String>,
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
