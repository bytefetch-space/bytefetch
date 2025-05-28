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
