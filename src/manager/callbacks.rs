use std::{sync::Arc, time::Duration};

use tokio::time::sleep;
use uuid::Uuid;

use crate::DownloadManager;

impl DownloadManager {
    pub fn on_progress<F>(&self, uuid: Uuid, callback: F, interval: Duration)
    where
        for<'a> F: Fn(u64) + Send + 'a,
    {
        let downloader = Arc::clone(self.downloads.get(&uuid).unwrap());
        tokio::spawn(async move {
            loop {
                callback(downloader.info.downloaded_bytes());
                sleep(interval).await;
            }
        });
    }
}
