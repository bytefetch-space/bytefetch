use std::sync::Arc;

use bytes::Bytes;
use reqwest::{Client, header::RANGE};
use tokio::sync::{
    Barrier,
    mpsc::{Sender, channel},
};

use super::{
    HttpDownloader,
    throttle::{ThrottleConfig, Throttler},
};

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
        let (sc, mut rc) = channel(100);
        let barrier = Arc::new(Barrier::new(self.config.threads_count as usize));

        for i in 0..self.config.threads_count {
            let part_range =
                HttpDownloader::extract_part_range(self.config.split_result.unwrap(), i as u64);
            let client = Arc::clone(&self.client);
            let raw_url = Arc::clone(&self.raw_url);
            let sc_clone = sc.clone();
            let throttle_config = Arc::clone(&self.config.throttle_config);
            let barrier = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                HttpDownloader::download_part(
                    client,
                    raw_url,
                    part_range,
                    throttle_config,
                    sc_clone,
                    barrier,
                )
                .await
            }));
        }

        while let Some(chunk) = rc.recv().await {
            self.info.add_to_downloaded_bytes(chunk.len() as u64);
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    async fn download_part(
        client: Arc<Client>,
        raw_url: Arc<String>,
        part_range: String,
        throttle_config: Arc<ThrottleConfig>,
        sc: Sender<Bytes>,
        barrier: Arc<Barrier>,
    ) {
        let mut response = client
            .get(raw_url.as_str())
            .header(RANGE, part_range)
            .send()
            .await
            .unwrap();

        let mut download_strategy = DownloadStrategy::new(sc.clone(), throttle_config.task_speed());

        while let Some(chunk) = response.chunk().await.unwrap() {
            download_strategy.handle_chunk(chunk).await;
            if throttle_config.has_throttle_changed() {
                download_strategy = DownloadStrategy::new(sc.clone(), throttle_config.task_speed());
                let wait_result = barrier.wait().await;
                if wait_result.is_leader() {
                    throttle_config.reset_has_throttle_changed();
                }
            }
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
    fn new(sc: Sender<Bytes>, task_speed: u64) -> Self {
        if task_speed > 0 {
            let throttle = Throttler::new(task_speed);
            DownloadStrategy::Throttled { sc, throttle }
        } else {
            DownloadStrategy::NotThrottled { sc }
        }
    }

    async fn handle_chunk(&mut self, chunk: Bytes) {
        match self {
            DownloadStrategy::NotThrottled { sc } => HttpDownloader::process_chunk(sc, chunk).await,
            DownloadStrategy::Throttled { sc, throttle } => {
                throttle.process_throttled(sc, chunk).await
            }
        }
    }
}
