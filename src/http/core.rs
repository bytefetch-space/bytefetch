use std::sync::Arc;

use bytes::Bytes;
use reqwest::{Client, header::RANGE};
use tokio::sync::mpsc::{Sender, channel};

use super::{HttpDownloader, throttle::Throttler};

impl HttpDownloader {
    fn extract_part_range((part_size, parts_before_decrease): (u64, u64), index: u64) -> String {
        let mut start = index * part_size;
        let mut end = (index + 1) * part_size - 1;
        if index > parts_before_decrease {
            start -= index - parts_before_decrease;
        }
        if index >= parts_before_decrease {
            end -= index - parts_before_decrease + 1;
        }
        format!("bytes={}-{}", start, end)
    }

    pub async fn start(&self) {
        let mut handles = vec![];
        let throttle_speed: Option<u64> = None;
        let (sc, rc) = channel(100);

        for i in 0..self.config.threads_count {
            let part_range =
                HttpDownloader::extract_part_range(self.config.split_result.unwrap(), i as u64);
            let client = Arc::clone(&self.client);
            let raw_url = Arc::clone(&self.raw_url);
            let sc_clone = sc.clone();
            let task_speed = throttle_speed.unwrap_or_default() / self.config.threads_count as u64;
            handles.push(tokio::spawn(async move {
                HttpDownloader::download_part(client, raw_url, part_range, task_speed, sc_clone)
                    .await
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    async fn download_part(
        client: Arc<Client>,
        raw_url: Arc<String>,
        part_range: String,
        task_speed: u64,
        sc: Sender<Bytes>,
    ) {
        let mut response = client
            .get(raw_url.as_str())
            .header(RANGE, part_range)
            .send()
            .await
            .unwrap();

        let mut download_strategy;
        if task_speed > 0 {
            let throttle = Throttler::new(task_speed);
            download_strategy = DownloadStrategy::Throttled {
                sc: sc.clone(),
                throttle,
            }
        } else {
            download_strategy = DownloadStrategy::NotThrottled { sc: sc.clone() }
        }

        while let Some(chunk) = response.chunk().await.unwrap() {
            download_strategy.handle_chunk(chunk).await;
        }
    }

    async fn process_chunk(sc: &mut Sender<Bytes>, chunk: Bytes) {
        sc.send(chunk).await.unwrap();
    }
}

enum DownloadStrategy {
    NotThrottled {
        sc: Sender<Bytes>,
    },
    Throttled {
        sc: Sender<Bytes>,
        throttle: Throttler,
    },
}

impl DownloadStrategy {
    async fn handle_chunk(&mut self, chunk: Bytes) {
        match self {
            DownloadStrategy::NotThrottled { sc } => HttpDownloader::process_chunk(sc, chunk).await,
            DownloadStrategy::Throttled { sc, throttle } => {
                throttle.process_throttled(sc, chunk).await
            }
        }
    }
}
