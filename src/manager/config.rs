use std::{path::PathBuf, time::Duration};

#[derive(Clone)]
pub struct DownloadConfig {
    pub timeout: Option<Duration>,
    pub directory: Option<PathBuf>,
    pub speed_limit: Option<u64>,
}
