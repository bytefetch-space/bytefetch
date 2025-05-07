use reqwest::Client;

pub struct HttpDownloaderSetupBuilder {
    client: Option<Client>,
    raw_url: Option<String>,
}

impl HttpDownloaderSetupBuilder {
    pub(super) fn default() -> Self {
        Self {
            client: None,
            raw_url: None,
        }
    }

    pub fn client(mut self, client: Client) -> Self {
        self.client = Some(client);
        self
    }

    pub fn url(mut self, raw_url: &str) -> Self {
        self.raw_url = Some(raw_url.to_string());
        self
    }
}
