mod filename_utils;
mod setup;

use setup::{ClientRequired, HttpDownloaderSetupBuilder};

pub struct HttpDownloader {
    pub filename: String,
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }
}
