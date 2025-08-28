use crate::http::{
    DownloadHandle, Error, HttpDownloadMode, builder_utils, config::HttpDownloadConfig,
    request_utils::RequestBuilderExt,
};

use super::{HttpDownloader, HttpDownloaderSetupErrors, info::HttpDownloadInfo};

use reqwest::{
    Client,
    header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH},
};
use std::{marker::PhantomData, sync::Arc, time::Duration};
use tokio_util::sync::CancellationToken;

pub struct ClientRequired;
pub struct UrlRequired;
pub struct SetupBuilder;
pub struct HttpDownloaderSetupBuilder<State = SetupBuilder> {
    client: Option<Client>,
    raw_url: Option<String>,
    tasks_count: Option<u8>,
    throttle_speed: Option<u64>,
    state: PhantomData<State>,
    timeout: Option<Duration>,
    token: Option<CancellationToken>,
}

impl HttpDownloaderSetupBuilder<ClientRequired> {
    pub fn client(mut self, client: Client) -> HttpDownloaderSetupBuilder<UrlRequired> {
        self.client = Some(client);
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            tasks_count: self.tasks_count,
            state: PhantomData::<UrlRequired>,
            throttle_speed: self.throttle_speed,
            timeout: self.timeout,
            token: self.token,
        }
    }
}

impl HttpDownloaderSetupBuilder<UrlRequired> {
    pub fn url(mut self, raw_url: &str) -> HttpDownloaderSetupBuilder<SetupBuilder> {
        self.raw_url = Some(raw_url.to_string());
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            tasks_count: self.tasks_count,
            state: PhantomData::<SetupBuilder>,
            throttle_speed: self.throttle_speed,
            timeout: self.timeout,
            token: self.token,
        }
    }
}

impl HttpDownloaderSetupBuilder {
    pub(super) fn default() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::<ClientRequired> {
            client: None,
            raw_url: None,
            tasks_count: None,
            state: PhantomData::<ClientRequired>,
            throttle_speed: None,
            timeout: None,
            token: None,
        }
    }

    pub fn tasks_count(mut self, count: u8) -> Self {
        self.tasks_count = Some(count);
        self
    }

    pub fn speed_limit(mut self, kilobytes_per_second: u64) -> Self {
        self.throttle_speed = Some(1024 * kilobytes_per_second);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn cancel_token(mut self, token: CancellationToken) -> Self {
        self.token = Some(token);
        self
    }

    fn generate_config(&self) -> Result<HttpDownloadConfig, HttpDownloaderSetupErrors> {
        Ok(HttpDownloadConfig::default()
            .try_set_tasks_count(self.tasks_count)?
            .set_throttle_speed(self.throttle_speed)
            .set_timeout(self.timeout))
    }

    pub fn build(self) -> Result<HttpDownloaderSetup, HttpDownloaderSetupErrors> {
        let config = self.generate_config()?;
        Ok(HttpDownloaderSetup {
            client: self.client.unwrap(),
            raw_url: self.raw_url.unwrap(),
            config,
            token: self.token,
        })
    }
}

pub struct HttpDownloaderSetup {
    client: Client,
    raw_url: String,
    config: HttpDownloadConfig,
    token: Option<CancellationToken>,
}

impl HttpDownloaderSetup {
    async fn get_headers(&self) -> Result<reqwest::Response, Error> {
        self.client
            .head(&self.raw_url)
            .send_with_timeout(self.config.timeout)
            .await
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

    fn generate_byte_ranges(
        config: &HttpDownloadConfig,
        mode: &HttpDownloadMode,
    ) -> Vec<(u64, u64)> {
        match mode {
            HttpDownloadMode::NonResumable => vec![],
            HttpDownloadMode::ResumableStream => vec![(0, 0)],
            HttpDownloadMode::ResumableMultithread => {
                let mut byte_ranges = vec![];
                let split_content = config.split_result.unwrap();
                for index in 0..config.tasks_count as u64 {
                    byte_ranges.push(builder_utils::calculate_part_range(split_content, index));
                }
                byte_ranges
            }
        }
    }

    pub async fn init(self) -> Result<HttpDownloader, Error> {
        let headers_response = self.get_headers().await?;
        let info = self.generate_info(headers_response);
        let mode = builder_utils::determine_mode(self.config.tasks_count, &info);

        let mut config = self.config;
        (mode == HttpDownloadMode::NonResumable).then(|| config.tasks_count = 0);
        config.split_result =
            builder_utils::try_split_content(&mode, &info.content_length(), config.tasks_count);
        Ok(HttpDownloader {
            client: Arc::new(self.client),
            raw_url: Arc::new(self.raw_url),
            info,
            byte_ranges: HttpDownloaderSetup::generate_byte_ranges(&config, &mode),
            mode,
            config,
            handle: Arc::new(DownloadHandle::new(
                self.token.unwrap_or(CancellationToken::new()),
            )),
        })
    }
}
