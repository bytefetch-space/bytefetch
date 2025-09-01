use std::{path::PathBuf, time::Duration};

use tokio_util::sync::CancellationToken;

use crate::http::{from_state::HttpDownloaderFromStateBuilder, setup::HttpDownloaderSetupBuilder};

pub(super) struct DownloadOptions {
    pub(super) timeout: Option<Duration>,
    pub(super) directory: Option<PathBuf>,
    pub(super) token: CancellationToken,
    pub(super) throttle_speed: Option<u64>,
}

impl DownloadOptions {
    pub(super) fn default() -> Self {
        Self {
            timeout: None,
            token: CancellationToken::new(),
            directory: None,
            throttle_speed: None,
        }
    }
}

macro_rules! impl_common_options {
    ($t:ty) => {
        impl $t {
            pub fn timeout(mut self, timeout: Duration) -> Self {
                self.options.timeout = Some(timeout);
                self
            }

            pub fn cancel_token(mut self, token: CancellationToken) -> Self {
                self.options.token = token;
                self
            }

            pub fn directory(mut self, path: PathBuf) -> Self {
                self.options.directory = Some(path);
                self
            }

            pub fn speed_limit(mut self, kilobytes_per_second: u64) -> Self {
                self.options.throttle_speed = Some(1024 * kilobytes_per_second);
                self
            }
        }
    };
}

impl_common_options!(HttpDownloaderSetupBuilder);
impl_common_options!(HttpDownloaderFromStateBuilder);
