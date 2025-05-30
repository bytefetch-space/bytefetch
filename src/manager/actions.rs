use std::sync::Arc;
use uuid::Uuid;

use crate::HttpDownloader;

use super::DownloadManager;

impl DownloadManager {
    pub fn add_download(&mut self, uuid: Uuid, downloader: HttpDownloader) {
        self.downloads.insert(uuid, Arc::new(downloader));
    }

    pub fn start_download(&self, uuid: Uuid) {
        let downloader = Arc::clone(self.downloads.get(&uuid).unwrap());

        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            rt.block_on(async {
                downloader.start().await;
            });
        });
    }
}
