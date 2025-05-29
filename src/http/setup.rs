use crate::http::config::HttpDownloadConfig;

use super::{HttpDownloadMode, HttpDownloader, HttpDownloaderSetupErrors, info::HttpDownloadInfo};

use reqwest::{
    Client,
    header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH},
};
use std::{marker::PhantomData, sync::Arc};

pub struct ClientRequired;
pub struct UrlRequired;
pub struct SetupBuilder;
pub struct HttpDownloaderSetupBuilder<State = SetupBuilder> {
    client: Option<Client>,
    raw_url: Option<String>,
    threads_count: Option<u8>,
    throttle_speed: Option<u64>,
    state: PhantomData<State>,
}

impl HttpDownloaderSetupBuilder<ClientRequired> {
    pub fn client(mut self, client: Client) -> HttpDownloaderSetupBuilder<UrlRequired> {
        self.client = Some(client);
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            threads_count: self.threads_count,
            state: PhantomData::<UrlRequired>,
            throttle_speed: self.throttle_speed,
        }
    }
}

impl HttpDownloaderSetupBuilder<UrlRequired> {
    pub fn url(mut self, raw_url: &str) -> HttpDownloaderSetupBuilder<SetupBuilder> {
        self.raw_url = Some(raw_url.to_string());
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            threads_count: self.threads_count,
            state: PhantomData::<SetupBuilder>,
            throttle_speed: self.throttle_speed,
        }
    }
}

impl HttpDownloaderSetupBuilder {
    pub(super) fn default() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::<ClientRequired> {
            client: None,
            raw_url: None,
            threads_count: None,
            state: PhantomData::<ClientRequired>,
            throttle_speed: None,
        }
    }

    pub fn threads_count(mut self, count: u8) -> Self {
        self.threads_count = Some(count);
        self
    }

    pub fn speed_limit(mut self, kilobytes_per_second: u64) -> Self {
        self.throttle_speed = Some(1024 * kilobytes_per_second);
        self
    }

    fn generate_config(&self) -> Result<HttpDownloadConfig, HttpDownloaderSetupErrors> {
        Ok(HttpDownloadConfig::default()
            .set_thread_count(self.threads_count)?
            .set_throttle_speed(self.throttle_speed))
    }

    pub fn build(self) -> Result<HttpDownloaderSetup, HttpDownloaderSetupErrors> {
        let config = self.generate_config()?;
        Ok(HttpDownloaderSetup {
            client: self.client.unwrap(),
            raw_url: self.raw_url.unwrap(),
            config,
        })
    }
}

pub struct HttpDownloaderSetup {
    client: Client,
    raw_url: String,
    config: HttpDownloadConfig,
}

impl HttpDownloaderSetup {
    async fn get_headers(&self) -> Result<reqwest::Response, reqwest::Error> {
        self.client.head(&self.raw_url).send().await
    }

    fn generate_info(&self, headers_response: reqwest::Response) -> HttpDownloadInfo {
        let content_disposition = &headers_response.headers().get(CONTENT_DISPOSITION);
        let content_length = &headers_response.headers().get(CONTENT_LENGTH);
        let accept_ranges = &headers_response.headers().get(ACCEPT_RANGES);
        HttpDownloadInfo::default()
            .extract_and_set_filename(&self.raw_url, content_disposition)
            .extract_and_set_content_length(content_length)
            .extract_and_set_is_resumable(accept_ranges)
    }

    fn determine_mode(&self, info: &HttpDownloadInfo) -> HttpDownloadMode {
        match (
            self.config.threads_count,
            info.content_length(),
            info.is_resumable(),
        ) {
            (_, _, false) => return HttpDownloadMode::NonResumable,
            (_, None, true) | (1, _, true) => return HttpDownloadMode::ResumableStream,
            (_, _, true) => return HttpDownloadMode::ResumableMultithread,
        }
    }

    fn split_content(content_length: u64, thread_number: u64) -> (u64, u64) {
        let mut remainder = content_length % thread_number;
        let mut part_size = content_length / thread_number;
        if remainder > 0 {
            part_size += 1
        } else {
            remainder = thread_number
        }
        (part_size, remainder) // Example: split_content(1003, 4) returns (251, 3), meaning 3 parts are 251 bytes and 1 part is 250 bytes
    }

    fn try_split_content(
        mode: &HttpDownloadMode,
        content_length: &Option<u64>,
        threads_count: u8,
    ) -> Option<(u64, u64)> {
        if *mode == HttpDownloadMode::NonResumable || *mode == HttpDownloadMode::ResumableStream {
            return None;
        }
        Some(HttpDownloaderSetup::split_content(
            content_length.unwrap(),
            threads_count as u64,
        ))
    }

    pub async fn init(self) -> HttpDownloader {
        let headers_response = self.get_headers().await.unwrap();
        let info = self.generate_info(headers_response);
        let mode = self.determine_mode(&info);
        let mut config = self.config;
        config.split_result = HttpDownloaderSetup::try_split_content(
            &mode,
            info.content_length(),
            config.threads_count,
        );
        HttpDownloader {
            client: Arc::new(self.client),
            raw_url: Arc::new(self.raw_url),
            info,
            mode,
            config,
        }
    }
}
