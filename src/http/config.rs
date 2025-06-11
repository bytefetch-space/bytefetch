use std::sync::Arc;

use super::{HttpDownloaderSetupErrors, throttle::ThrottleConfig};

const DEFAULT_TASKS_COUNT: u8 = 8;
const MIN_TASKS_COUNT: u8 = 1;
const MAX_TASKS_COUNT: u8 = 64;

pub(super) struct HttpDownloadConfig {
    pub(super) tasks_count: u8,
    pub(super) split_result: Option<(u64, u64)>,
    pub(super) throttle_config: Arc<ThrottleConfig>,
    pub(super) is_new: bool,
}

impl HttpDownloadConfig {
    pub(super) fn default() -> Self {
        Self {
            tasks_count: 0,
            split_result: None,
            throttle_config: Arc::new(ThrottleConfig::default()),
            is_new: true,
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

    pub(super) fn mark_resumed(mut self) -> Self {
        self.is_new = false;
        self
    }
}
