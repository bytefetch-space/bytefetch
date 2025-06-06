use std::sync::Arc;

use super::{HttpDownloaderSetupErrors, throttle::ThrottleConfig};

const DEFAULT_THREADS_COUNT: u8 = 8;
const MIN_THREADS_COUNT: u8 = 1;
const MAX_THREADS_COUNT: u8 = 64;

pub(super) struct HttpDownloadConfig {
    pub(super) threads_count: u8,
    pub(super) split_result: Option<(u64, u64)>,
    pub(super) throttle_config: Arc<ThrottleConfig>,
    pub(super) is_new: bool,
}

impl HttpDownloadConfig {
    pub(super) fn default() -> Self {
        Self {
            threads_count: 0,
            split_result: None,
            throttle_config: Arc::new(ThrottleConfig::default()),
            is_new: true,
        }
    }

    pub(super) fn set_thread_count(
        mut self,
        thread_count: Option<u8>,
    ) -> Result<Self, HttpDownloaderSetupErrors> {
        self.threads_count = match thread_count {
            Some(v) if v >= MIN_THREADS_COUNT && v <= MAX_THREADS_COUNT => v,
            Some(_) => return Err(HttpDownloaderSetupErrors::InvalidThreadsCount),
            None => DEFAULT_THREADS_COUNT,
        };
        Ok(self)
    }

    pub(super) fn set_throttle_speed(self, throttle_speed: Option<u64>) -> Self {
        let task_speed = throttle_speed.unwrap_or_default() / self.threads_count as u64;
        self.throttle_config.set_task_speed(task_speed);
        self
    }

    pub(super) fn mark_resumed(mut self) -> Self {
        self.is_new = false;
        self
    }
}
