use tracing::debug;

use crate::domain::Provider;

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: Provider,
    pub cookie: String,
    pub access_token: String,
}

impl ProviderConfig {
    pub fn new(provider_type: Provider, cookie: String, access_token: String) -> Self {
        debug!("Configuring {} payment verification", provider_type);

        ProviderConfig {
            provider_type,
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
