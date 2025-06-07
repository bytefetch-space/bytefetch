use std::sync::Arc;
use uuid::Uuid;

use crate::HttpDownloader;

use super::DownloadManager;

impl DownloadManager {
    fn add_download(&mut self, uuid: Uuid, downloader: HttpDownloader) {
        self.downloads.insert(uuid, Arc::new(downloader));
    }

    fn start_download(&self, uuid: Uuid) {
        let downloader = Arc::clone(self.downloads.get(&uuid).unwrap());
    }
}
