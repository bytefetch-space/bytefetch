use std::{path::PathBuf, time::Duration};

use tokio_util::sync::CancellationToken;

use crate::http::{from_state::HttpDownloaderFromStateBuilder, setup::HttpDownloaderSetupBuilder};

pub(crate) struct DownloadOptions {
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

pub trait CommonDownloadOptions: Sized {
    fn options_mut(&mut self) -> &mut DownloadOptions;

    fn timeout(mut self, timeout: Duration) -> Self {
        self.options_mut().timeout = Some(timeout);
        self
    }
    fn cancel_token(mut self, token: CancellationToken) -> Self {
        self.options_mut().token = token;
        self
    }

    fn directory(mut self, path: PathBuf) -> Self {
        self.options_mut().directory = Some(path);
        self
    }

    fn speed_limit(mut self, kilobytes_per_second: u64) -> Self {
        self.options_mut().throttle_speed = Some(1024 * kilobytes_per_second);
        self
    }
}

macro_rules! impl_download_options {
    ($t:ty) => {
        macro_rules! delegate {
            ($fn_name:ident, $arg_ty:ty) => {
                pub fn $fn_name(self, value: $arg_ty) -> Self {
                    <$t as CommonDownloadOptions>::$fn_name(self, value)
                }
            };
        }

        impl $t {
            delegate!(timeout, Duration);
            delegate!(cancel_token, CancellationToken);
            delegate!(directory, PathBuf);
            delegate!(speed_limit, u64);
        }

        impl CommonDownloadOptions for $t {
            fn options_mut(&mut self) -> &mut DownloadOptions {
                &mut self.options
            }
        }
    };
}

impl_download_options!(HttpDownloaderSetupBuilder);
impl_download_options!(HttpDownloaderFromStateBuilder);
