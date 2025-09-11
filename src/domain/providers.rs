use anyhow::Error;
use clap::Parser;
use tracing::debug;

use crate::domain::Provider;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ProviderArgs {
    #[clap(long)]
    pub provider: Provider,
}

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: Provider,
    pub profile_id: Option<String>,
    pub transaction_id: String,
    pub cookie: String,
    pub access_token: String,
}

impl ProviderConfig {
    pub fn new(
        provider_type: Provider,
        profile_id: Option<String>,
        transaction_id: String,
        cookie: String,
        access_token: String,
    ) -> Self {
        debug!("Configuring {} payment verification for transaction {}", provider_type, transaction_id);
        if let (Provider::Wise, Some(pid)) = (&provider_type, &profile_id) {
            debug!("Using Wise profile ID: {}", pid);
        }

        ProviderConfig {
            provider_type,
            profile_id,
            transaction_id,
            cookie,
            access_token,
        }
    }

    pub fn auth_headers(&self) -> Vec<(&str, &str)> {
        vec![
            ("Cookie", self.cookie.as_str()),
            ("X-Access-Token", self.access_token.as_str()),
        ]
    }

    pub fn transaction_endpoint(&self) -> Result<String, Error> {
        match self.provider_type {
            Provider::Wise => match &self.profile_id {
                Some(profile_id) => Ok(format!(
                    "/gateway/v3/profiles/{}/transfers/{}",
                    profile_id, self.transaction_id
                )),
                None => Err(anyhow::anyhow!(
                    "Profile ID is required for Wise transaction endpoint"
                )),
            },
            Provider::PayPal => Ok(format!(
                "/myaccount/activities/details/inline/{}",
                self.transaction_id
            )),
        }
    }
}
