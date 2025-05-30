mod bytes_aggregator;
mod config;
mod core;
mod filename_utils;
mod info;
mod setup;
#[cfg(test)]
mod tests;
mod throttle;

use std::sync::Arc;

use config::HttpDownloadConfig;
use info::HttpDownloadInfo;
use reqwest::Client;
use setup::{ClientRequired, HttpDownloaderSetupBuilder};

pub struct HttpDownloader {
    client: Arc<Client>,
    raw_url: Arc<String>,
    pub info: HttpDownloadInfo,
    pub mode: HttpDownloadMode,
    config: HttpDownloadConfig,
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }

    pub fn mode(self) -> HttpDownloadMode {
        self.mode
    }

    pub fn change_speed_limit(&self, kilobytes_per_second: Option<u64>) {
        let throttle_speed = if kilobytes_per_second == None {
            None
        } else {
            Some(kilobytes_per_second.unwrap() * 1024)
        };

        self.config
            .throttle_config
            .change_throttle_speed(throttle_speed, self.config.threads_count as u64);
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpDownloadMode {
    NonResumable,
    ResumableStream,
    ResumableMultithread,
}

#[derive(Debug)]
pub enum HttpDownloaderSetupErrors {
    InvalidThreadsCount,
}
