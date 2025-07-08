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
use reqwest::Client;
use setup::HttpDownloaderSetupBuilder;
use std::sync::{Arc, Mutex};

pub struct HttpDownloader {
    client: Arc<Client>,
    raw_url: Arc<String>,
    pub info: HttpDownloadInfo,
    pub mode: HttpDownloadMode,
    config: HttpDownloadConfig,
    byte_ranges: Vec<(u64, u64)>,
    status: Mutex<Status>,
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

    fn set_status(&self, status: Status) {
        *self.status.lock().unwrap() = status
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

pub enum Status {
    Pending,
    Downloading,
    Completed,
}
