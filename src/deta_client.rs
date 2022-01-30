/// Stores the necessary information for deta integration.
/// Check [deta docs](https://docs.deta.sh/docs/home/) for more information.
pub struct DetaClient {
    api_key: String,
}

impl DetaClient {
    /// Creates an `DetaClient` instance.
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_owned(),
        }
    }

    /// Returns api key.
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Returns project id.
    pub fn project_id(&self) -> &str {
        &self.api_key.split('_').next().unwrap()
    }
}
