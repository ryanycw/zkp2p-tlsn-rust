use crate::domain::ProviderType;

#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub provider_type: ProviderType,
    pub profile_id: Option<String>,
    pub transaction_id: String,
    pub cookie: String,
    pub access_token: String,
}

impl ProviderConfig {
    pub fn new(
        provider_type: ProviderType,
        profile_id: Option<String>,
        transaction_id: String,
        cookie: String,
        access_token: String,
    ) -> Self {
        match provider_type {
            ProviderType::Wise => {
                println!("🔐 Configuring ZKP2P Wise payment verification:");
                if let Some(ref pid) = profile_id {
                    println!("   Profile ID: {}", pid);
                }
            }
            ProviderType::PayPal => {
                println!("🔐 Configuring ZKP2P PayPal payment verification:");
            }
        }
        println!("   Target Payment ID: {}", transaction_id);
        println!("   Session credentials will remain private in all proofs");

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

    pub fn transaction_endpoint(&self) -> Result<String, &'static str> {
        match self.provider_type {
            ProviderType::Wise => match &self.profile_id {
                Some(profile_id) => Ok(format!(
                    "/gateway/v3/profiles/{}/transfers/{}",
                    profile_id, self.transaction_id
                )),
                None => Err("Profile ID is required for Wise transaction endpoint"),
            },
            ProviderType::PayPal => Ok(format!(
                "/myaccount/activities/details/inline/{}",
                self.transaction_id
            )),
        }
    }
}
