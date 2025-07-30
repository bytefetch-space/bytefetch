use std::sync::atomic::{AtomicU64, Ordering};

use reqwest::header::HeaderValue;

use super::filename_utils;

#[derive(Debug)]
pub struct HttpDownloadInfo {
    filename: String,
    content_length: Option<u64>,
    is_resumable: bool,
    downloaded_bytes: AtomicU64,
}

impl HttpDownloadInfo {
    pub(super) fn default() -> Self {
        Self {
            filename: String::new(),
            content_length: None,
            is_resumable: false,
            downloaded_bytes: AtomicU64::new(0),
        }
    }

    pub(super) fn extract_and_set_filename(
        mut self,
        raw_url: &str,
        content_disposition: &Option<&HeaderValue>,
    ) -> Self {
        self.filename = filename_utils::extract_filename(raw_url, content_disposition);
        self
    }

    pub(super) fn set_filename(mut self, filename: String) -> Self {
        self.filename = filename;
        self
    }

    pub(super) fn extract_and_set_content_length(
        mut self,
        content_length: &Option<&HeaderValue>,
    ) -> Self {
        self.content_length = content_length
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
        self
    }

    pub(super) fn set_content_length(mut self, content_length: Option<u64>) -> Self {
        self.content_length = content_length;
        self
    }

    pub(super) fn extract_and_set_is_resumable(
        mut self,
        accept_ranges: &Option<&HeaderValue>,
    ) -> Self {
        self.is_resumable = match accept_ranges {
            Some(value) => value.to_str().map_or(false, |s| s == "bytes"),
            None => false,
        };
        self
    }

    pub(super) fn set_is_resumable(mut self, is_resumable: bool) -> Self {
        self.is_resumable = is_resumable;
        self
    }

    pub(super) fn add_to_downloaded_bytes(&self, number: u64) {
        self.downloaded_bytes.fetch_add(number, Ordering::Relaxed);
    }
}

impl HttpDownloadInfo {
    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn content_length(&self) -> Option<u64> {
        self.content_length
    }

    pub fn is_resumable(&self) -> bool {
        self.is_resumable
    }

    pub fn downloaded_bytes(&self) -> u64 {
        self.downloaded_bytes.load(Ordering::Relaxed)
    }
}
