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
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }
}
