mod filename_utils;
mod setup;
#[cfg(test)]
mod tests;

use setup::{ClientRequired, HttpDownloaderSetupBuilder};

pub struct HttpDownloader {
    pub filename: String,
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }
}
