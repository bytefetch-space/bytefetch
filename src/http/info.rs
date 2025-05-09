use reqwest::header::HeaderValue;

use super::filename_utils;

#[derive(Debug)]
pub struct HttpDownloadInfo {
    filename: String,
    content_length: Option<u64>,
}

impl HttpDownloadInfo {
    pub(crate) fn default() -> Self {
        Self {
            filename: String::new(),
            content_length: None,
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

    pub(crate) fn extract_and_set_content_length(
        mut self,
        content_length: &Option<&HeaderValue>,
    ) -> Self {
        self.content_length = content_length
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok());
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
}
