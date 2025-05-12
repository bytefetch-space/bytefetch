mod config;
mod filename_utils;
mod info;
mod setup;
#[cfg(test)]
mod tests;

use info::HttpDownloadInfo;
use setup::{ClientRequired, HttpDownloaderSetupBuilder};

pub struct HttpDownloader {
    pub info: HttpDownloadInfo,
    pub mode: HttpDownloadMode,
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }

    pub fn mode(self) -> HttpDownloadMode {
        self.mode
    }
}

#[derive(Debug)]
pub enum HttpDownloadMode {
    NonResumable,
    ResumableStream,
    ResumableMultithread,
}

#[derive(Debug)]
pub enum HttpDownloaderSetupErrors {
    InvalidThreadsCount,
}
