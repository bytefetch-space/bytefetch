use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::{HttpDownloader, Status, manager::Callbacks};

use super::DownloadManager;
use std::{hash::Hash, sync::Arc, time::Duration};

macro_rules! call_cb {
    ($cb:expr, $($arg:expr),*) => {
        if let Some(cb) = $cb {
            cb($($arg),*);
        }
    }
}

impl<T> DownloadManager<T>
where
    T: Hash + Eq + Clone + Send + 'static,
{
    pub fn add_download(&self, key: T, url: &str) {
        self.urls
            .lock()
            .insert(key.clone(), Arc::new(String::from(url)));
        self.tokens.lock().insert(key, CancellationToken::new());
    }

    pub fn start_download(&self, key: T) {
        let callbacks = self.callbacks.clone();

        let url = self.urls.lock().get(&key).unwrap().clone();
        let client = reqwest::Client::builder().build().unwrap();
        let setup = HttpDownloader::setup()
            .client(client)
            .url(&url)
            .tasks_count(16)
            .cancel_token(self.tokens.lock().get(&key).unwrap().clone())
            .build()
            .unwrap();

        self.runtime.spawn(async move {
            let downloader = match setup.init().await {
                Ok(downloader) => Arc::new(downloader),
                Err(err) => {
                    call_cb!(&callbacks.on_failed, key, err);
                    return;
                }
            };
            let download_task = Arc::clone(&downloader);
            tokio::spawn(Self::monitor_download(key, downloader, callbacks));
            download_task.start().await;
        });
    }

    async fn monitor_download(
        key: T,
        downloader: Arc<HttpDownloader>,
        callbacks: Arc<Callbacks<T>>,
    ) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));
        let mut last = downloader.info.downloaded_bytes();
        let alpha = 0.4;
        let mut last_instant_speed: f64 = 0.0;
        let mut ema_speed: f64 = 0.0;
        let mut idle_ticks: u8 = 0;

        loop {
            select! {
                _ = interval.tick() => {
                    if let Some(cb) = &callbacks.on_progress {
                        let current = downloader.info.downloaded_bytes();
                        let instant_speed = (current - last) as f64;
                        let sma_speed = (instant_speed + last_instant_speed) * 0.5;

                        idle_ticks = if sma_speed == 0.0 { idle_ticks + 1 } else { 0 };
                        ema_speed = if idle_ticks == 3 {
                            0.0
                        } else {
                            if ema_speed == 0.0 {
                                sma_speed
                            } else {
                                alpha * sma_speed + (1.0 - alpha) * ema_speed
                            }
                        };

                        cb(key.clone(), current, ema_speed as u64);
                        last = current;
                        last_instant_speed = instant_speed;
                    }
                }

                _ = downloader.wait_until_finished() => {
                    break
                }
            }
        }

        if let Some(cb) = &callbacks.on_progress {
            cb(
                key.clone(),
                downloader.info.downloaded_bytes(),
                downloader.info.downloaded_bytes() - last,
            )
        }

        match downloader.status() {
            Status::Completed => call_cb!(&callbacks.on_completed, key),
            Status::Failed(err) => call_cb!(&callbacks.on_failed, key, err),
            Status::Canceled => call_cb!(&callbacks.on_canceled, key),
            _ => {}
        }
    }

    pub fn cancel_download(&self, key: T) {
        if let Some(token) = self.tokens.lock().get(&key) {
            token.cancel();
        }
    }
}
