use reqwest::header::HeaderValue;

use super::filename_utils;

#[derive(Debug)]
pub struct HttpDownloadInfo {
    filename: String,
    content_length: Option<u64>,
    is_resumable: bool,
}

impl HttpDownloadInfo {
    pub(super) fn default() -> Self {
        Self {
            filename: String::new(),
            content_length: None,
            is_resumable: false,
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

    pub(super) fn extract_and_set_content_length(
        mut self,
        content_length: &Option<&HeaderValue>,
    ) -> Self {
        self.content_length = content_length
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
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
}

impl HttpDownloadInfo {
    pub fn filename(&self) -> &str {
        &self.filename
    }

    pub fn content_length(&self) -> &Option<u64> {
        &self.content_length
    }

    pub fn is_resumable(&self) -> bool {
        self.is_resumable
    }
}
