mod setup;

use setup::{ClientRequired, HttpDownloaderSetupBuilder};

pub struct HttpDownloader {}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }
}
