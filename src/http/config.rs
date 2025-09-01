use std::{path::PathBuf, sync::Arc, time::Duration};

use super::{HttpDownloaderSetupErrors, throttle::ThrottleConfig};

const DEFAULT_TASKS_COUNT: u8 = 8;
const MIN_TASKS_COUNT: u8 = 1;
const MAX_TASKS_COUNT: u8 = 64;
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(10);

pub(super) struct HttpDownloadConfig {
    pub(super) tasks_count: u8,
    pub(super) split_result: Option<(u64, u64)>,
    pub(super) throttle_config: Arc<ThrottleConfig>,
    pub(super) is_new: bool,
    pub(super) timeout: Duration,
    pub(super) directory: PathBuf,
}

impl HttpDownloadConfig {
    pub(super) fn default() -> Self {
        Self {
            tasks_count: 0,
            split_result: None,
            throttle_config: Arc::new(ThrottleConfig::default()),
            is_new: true,
            timeout: DEFAULT_TIMEOUT,
            directory: PathBuf::new(),
        }
    }

    pub(super) fn try_set_tasks_count(
        mut self,
        tasks_count: Option<u8>,
    ) -> Result<Self, HttpDownloaderSetupErrors> {
        self.tasks_count = match tasks_count {
            Some(v) if v >= MIN_TASKS_COUNT && v <= MAX_TASKS_COUNT => v,
            Some(_) => return Err(HttpDownloaderSetupErrors::InvalidThreadsCount),
            None => DEFAULT_TASKS_COUNT,
        };
        Ok(self)
    }

    pub(super) fn set_tasks_count(mut self, task_count: u8) -> Self {
        self.tasks_count = task_count;
        self
    }

    pub(super) fn set_throttle_speed(self, throttle_speed: Option<u64>) -> Self {
        let task_speed = throttle_speed.unwrap_or_default() / self.tasks_count as u64;
        self.throttle_config.set_task_speed(task_speed);
        self
    }

    pub(super) fn set_timeout(mut self, timeout: Option<Duration>) -> Self {
        if let Some(t) = timeout {
            self.timeout = t;
        }
        self
    }

    pub(super) fn mark_resumed(mut self) -> Self {
        self.is_new = false;
        self
    }

    pub(super) fn try_set_directory(
        mut self,
        path: Option<PathBuf>,
    ) -> Result<Self, HttpDownloaderSetupErrors> {
        if let Some(path) = path {
            if path.is_dir() || path.as_os_str().is_empty() {
                self.directory = path;
            } else {
                return Err(HttpDownloaderSetupErrors::InvalidDirectory);
            }
        }
        Ok(self)
    }

    pub(super) fn set_directory(mut self, path: PathBuf) -> Self {
        self.directory = path;
        self
    }
}
