mod setup;

use setup::HttpDownloaderSetupBuilder;

pub struct HttpDownloader {}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder {
        HttpDownloaderSetupBuilder::default()
    }
}
