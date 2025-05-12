const DEFAULT_THREADS_COUNT: u8 = 8;

pub(super) struct HttpDownloadConfig {
    pub(super) threads_count: u8,
}

impl HttpDownloadConfig {
    pub(super) fn default() -> Self {
        Self { threads_count: 0 }
    }

    pub(super) fn set_thread_count(mut self, thread_count: Option<u8>) -> Self {
        self.threads_count = if let Some(value) = thread_count {
            value
        } else {
            DEFAULT_THREADS_COUNT
        };
        self
    }
}
