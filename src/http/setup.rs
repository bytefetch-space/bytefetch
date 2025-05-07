use super::{HttpDownloader, filename_utils};

use reqwest::{Client, header::CONTENT_DISPOSITION};
use std::marker::PhantomData;

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

    pub fn build(self) -> HttpDownloaderSetup {
        HttpDownloaderSetup {
            client: self.client.unwrap(),
            raw_url: self.raw_url.unwrap(),
        }
    }
}

pub struct HttpDownloaderSetup {
    client: Client,
    raw_url: String,
}

impl HttpDownloaderSetup {
    async fn get_headers(&self) -> Result<reqwest::Response, reqwest::Error> {
        self.client.head(&self.raw_url).send().await
    }

    pub async fn init(&self) -> HttpDownloader {
        let headers = self.get_headers().await.unwrap();
        let mut filename = filename_utils::extract_filename_from_header(
            &headers.headers().get(CONTENT_DISPOSITION),
        );
        if filename == None {
            filename = filename_utils::extract_filename_from_url(&self.raw_url);
        }
        HttpDownloader {
            filename: filename.unwrap(),
        }
    }
}
