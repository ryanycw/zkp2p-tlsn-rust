use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct AuthArgs {
    #[clap(long)]
    pub provider: String,
    #[clap(long)]
    pub profile_id: Option<String>,
    #[clap(long)]
    pub transaction_id: String,
    #[clap(long)]
    pub cookie: String,
    #[clap(long)]
    pub access_token: String,
}
