use super::{HttpDownloader, info::HttpDownloadInfo};

use reqwest::{
    Client,
    header::{CONTENT_DISPOSITION, CONTENT_LENGTH},
};
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

    fn generate_info(&self, headers_response: reqwest::Response) -> HttpDownloadInfo {
        let content_disposition = &headers_response.headers().get(CONTENT_DISPOSITION);
        let content_length = &headers_response.headers().get(CONTENT_LENGTH);
        HttpDownloadInfo::default()
            .extract_and_set_filename(&self.raw_url, content_disposition)
            .extract_and_set_content_length(content_length)
    }

    pub async fn init(&self) -> HttpDownloader {
        let headers_response = self.get_headers().await.unwrap();
        let info = self.generate_info(headers_response);
        HttpDownloader { info }
    }
}
