use tokio_util::sync::CancellationToken;

use crate::DownloadConfig;

#[derive(Clone)]
pub struct DownloadEntry {
    pub(super) url: String,
    pub(super) token: CancellationToken,
    pub(super) config: Option<DownloadConfig>,
}

impl DownloadEntry {
    fn new_entry(url: &str, config: Option<DownloadConfig>) -> Self {
        Self {
            url: String::from(url),
            token: CancellationToken::new(),
            config,
        }
    }

    pub fn new(url: &str, config: DownloadConfig) -> Self {
        Self::new_entry(url, Some(config))
    }

    pub fn new_default(url: &str) -> Self {
        Self::new_entry(url, None)
    }
}
