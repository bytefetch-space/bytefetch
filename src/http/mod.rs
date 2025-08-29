mod builder_utils;
mod bytes_aggregator;
mod config;
mod core;
mod file_writer;
mod filename_utils;
mod from_state;
mod info;
mod progress_state;
mod request_utils;
mod session;
mod setup;
#[cfg(test)]
mod tests;
mod throttle;

use crate::http::{from_state::HttpDownloaderFromStateBuilder, progress_state::ProgressState};
use config::HttpDownloadConfig;
use info::HttpDownloadInfo;
use parking_lot::Mutex;
use reqwest::Client;
use setup::HttpDownloaderSetupBuilder;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio_util::sync::CancellationToken;

pub struct HttpDownloader {
    client: Arc<Client>,
    raw_url: Arc<String>,
    pub info: HttpDownloadInfo,
    pub mode: HttpDownloadMode,
    config: HttpDownloadConfig,
    byte_ranges: Vec<(u64, u64)>,
    handle: Arc<DownloadHandle>,
}

impl HttpDownloader {
    pub fn setup() -> HttpDownloaderSetupBuilder<setup::ClientRequired> {
        HttpDownloaderSetupBuilder::default()
    }

    pub fn from_state(
        filename: &str,
    ) -> HttpDownloaderFromStateBuilder<from_state::ClientRequired> {
        HttpDownloaderFromStateBuilder::new(String::from(filename))
    }

    pub fn mode(self) -> HttpDownloadMode {
        self.mode
    }

    pub fn change_speed_limit(&self, kilobytes_per_second: Option<u64>) {
        let throttle_speed = if kilobytes_per_second == None {
            None
        } else {
            Some(kilobytes_per_second.unwrap() * 1024)
        };

        self.config
            .throttle_config
            .change_throttle_speed(throttle_speed, self.config.tasks_count as u64);
    }

    pub fn status(&self) -> Status {
        (*self.handle.effective_status.lock()).clone()
    }

    pub async fn wait_until_finished(&self) {
        self.handle.finished.notified().await
    }
}

#[derive(Debug, PartialEq)]
pub enum HttpDownloadMode {
    NonResumable,
    ResumableStream,
    ResumableMultithread,
}

#[derive(Debug)]
pub enum HttpDownloaderSetupErrors {
    InvalidThreadsCount,
}

struct DownloadHandle {
    raw_status: Mutex<Status>,
    effective_status: Mutex<Status>,
    token: CancellationToken,
    finished: Notify,
}

impl DownloadHandle {
    fn new(token: CancellationToken) -> Self {
        Self {
            raw_status: Mutex::new(Status::Pending),
            effective_status: Mutex::new(Status::Pending),
            token,
            finished: Notify::new(),
        }
    }

    fn mark_downloading(&self) {
        let mut raw_status = self.raw_status.lock();
        *raw_status = Status::Downloading;
        let mut effective_status = self.effective_status.lock();
        *effective_status = Status::Downloading;
    }

    fn update_if_downloading(&self, new_status: Status) {
        let mut raw_status = self.raw_status.lock();
        if let Status::Downloading = *raw_status {
            *raw_status = new_status;
            self.token.cancel();
        }
    }

    fn mark_canceled(&self) {
        self.update_if_downloading(Status::Canceled);
    }

    fn mark_failed<E: Into<Error>>(&self, err: E) {
        self.update_if_downloading(Status::Failed(err.into()));
    }

    fn mark_finished(&self) {
        let raw_status = self.raw_status.lock();
        let mut effective_status = self.effective_status.lock();
        match &*raw_status {
            Status::Downloading => *effective_status = Status::Completed,
            _ => *effective_status = (*raw_status).clone(),
        }
        self.finished.notify_waiters();
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    Network(Arc<reqwest::Error>),
    Io(Arc<std::io::Error>),
    Timeout,
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Network(Arc::new(err))
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(Arc::new(err))
    }
}

#[derive(Debug, Clone)]
pub enum Status {
    Pending,
    Downloading,
    Completed,
    Failed(Error),
    Canceled,
}
