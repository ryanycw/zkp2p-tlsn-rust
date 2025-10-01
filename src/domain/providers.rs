use tracing::debug;

use crate::domain::Provider;

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: Provider,
    pub transaction_id: String,
    pub cookie: String,
    pub access_token: String,
}

impl ProviderConfig {
    pub fn new(
        provider_type: Provider,
        transaction_id: String,
        cookie: String,
        access_token: String,
    ) -> Self {
        debug!(
            "Configuring {} payment verification for transaction {}",
            provider_type, transaction_id
        );

        ProviderConfig {
            provider_type,
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
}
