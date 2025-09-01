use crate::http::{
    BuilderErrors, DownloadHandle, Error, HttpDownloadMode, builder_utils,
    config::HttpDownloadConfig, options::DownloadOptions, request_utils::RequestBuilderExt,
};

use super::{HttpDownloader, info::HttpDownloadInfo};

use reqwest::{
    Client,
    header::{ACCEPT_RANGES, CONTENT_DISPOSITION, CONTENT_LENGTH, CONTENT_TYPE},
};
use std::{marker::PhantomData, sync::Arc};

pub struct ClientRequired;
pub struct UrlRequired;
pub struct SetupBuilder;
pub struct HttpDownloaderSetupBuilder<State = SetupBuilder> {
    client: Option<Client>,
    raw_url: Option<String>,
    tasks_count: Option<u8>,
    state: PhantomData<State>,
    pub(super) options: DownloadOptions,
}

impl HttpDownloaderSetupBuilder<ClientRequired> {
    pub fn client(mut self, client: Client) -> HttpDownloaderSetupBuilder<UrlRequired> {
        self.client = Some(client);
        HttpDownloaderSetupBuilder {
            client: self.client,
            raw_url: self.raw_url,
            tasks_count: self.tasks_count,
            state: PhantomData::<UrlRequired>,
            options: self.options,
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
            options: self.options,
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
            options: DownloadOptions::default(),
        }
    }

    pub fn tasks_count(mut self, count: u8) -> Self {
        self.tasks_count = Some(count);
        self
    }

    fn generate_config(&self) -> Result<HttpDownloadConfig, BuilderErrors> {
        Ok(HttpDownloadConfig::default()
            .try_set_tasks_count(self.tasks_count)?
            .try_set_directory(self.options.directory.clone())?
            .set_timeout(self.options.timeout))
    }

    pub fn build(self) -> Result<HttpDownloaderSetup, BuilderErrors> {
        let config = self.generate_config()?;
        Ok(HttpDownloaderSetup {
            client: self.client.unwrap(),
            raw_url: self.raw_url.unwrap(),
            config,
            options: self.options,
        })
    }
}

pub struct HttpDownloaderSetup {
    client: Client,
    raw_url: String,
    config: HttpDownloadConfig,
    options: DownloadOptions,
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
        let content_type = &headers_response.headers().get(CONTENT_TYPE);
        let content_length = &headers_response.headers().get(CONTENT_LENGTH);
        let accept_ranges = &headers_response.headers().get(ACCEPT_RANGES);
        HttpDownloadInfo::default()
            .extract_and_set_filename(&self.raw_url, content_disposition, content_type)
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
        config.set_throttle_speed(self.options.throttle_speed);

        config.split_result =
            builder_utils::try_split_content(&mode, &info.content_length(), config.tasks_count);
        Ok(HttpDownloader {
            client: Arc::new(self.client),
            raw_url: Arc::new(self.raw_url),
            info,
            byte_ranges: HttpDownloaderSetup::generate_byte_ranges(&config, &mode),
            mode,
            config,
            handle: Arc::new(DownloadHandle::new(self.options.token)),
        })
    }
}
