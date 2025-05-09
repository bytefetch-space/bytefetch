mod filename_utils;
mod setup;
#[cfg(test)]
mod tests;

use setup::{ClientRequired, HttpDownloaderSetupBuilder};

pub struct HttpDownloader {
    pub filename: String,
    pub content_length: Option<u64>,
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }
}
