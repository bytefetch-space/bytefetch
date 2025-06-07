mod actions;
mod callbacks;

use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::HttpDownloader;

struct DownloadManager {
    pub downloads: HashMap<Uuid, Arc<HttpDownloader>>,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            downloads: HashMap::new(),
        }
    }
}
