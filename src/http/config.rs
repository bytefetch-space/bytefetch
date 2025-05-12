use super::HttpDownloaderSetupErrors;

const DEFAULT_THREADS_COUNT: u8 = 8;
const MIN_THREADS_COUNT: u8 = 1;
const MAX_THREADS_COUNT: u8 = 64;

pub(super) struct HttpDownloadConfig {
    pub(super) threads_count: u8,
}

impl HttpDownloadConfig {
    pub(super) fn default() -> Self {
        Self { threads_count: 0 }
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
}
