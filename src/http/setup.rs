use std::marker::PhantomData;

use super::HttpDownloader;

use reqwest::Client;

pub struct ClientRequired;
pub struct UrlRequired;
pub struct SetupBuilder;
pub struct HttpDownloaderSetupBuilder<State = SetupBuilder> {
    client: Option<Client>,
    raw_url: Option<String>,
    state: PhantomData<State>,
}

impl HttpDownloaderSetupBuilder<ClientRequired> {
    pub fn client(mut self, client: Client) -> HttpDownloaderSetupBuilder<UrlRequired> {
        self.client = Some(client);
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            state: PhantomData::<UrlRequired>,
        }
    }
}

impl HttpDownloaderSetupBuilder<UrlRequired> {
    pub fn url(mut self, raw_url: &str) -> HttpDownloaderSetupBuilder<SetupBuilder> {
        self.raw_url = Some(raw_url.to_string());
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            state: PhantomData::<SetupBuilder>,
        }
    }
}

impl HttpDownloaderSetupBuilder {
    pub(super) fn default() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::<ClientRequired> {
            client: None,
            raw_url: None,
            state: PhantomData::<ClientRequired>,
        }
    }

    pub fn build(&self) -> HttpDownloaderSetup {
        HttpDownloaderSetup {}
    }
}

pub struct HttpDownloaderSetup {}

impl HttpDownloaderSetup {
    pub fn init(&self) -> HttpDownloader {
        HttpDownloader {}
    }
}
