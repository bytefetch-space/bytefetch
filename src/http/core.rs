use std::{sync::Arc, vec};

use bytes::Bytes;
use reqwest::{Client, header::RANGE};
use tokio::sync::{
    Barrier,
    mpsc::{Sender, channel},
};

use super::{
    HttpDownloader,
    bytes_aggregator::BytesAggregator,
    throttle::{ThrottleConfig, Throttler},
};

impl HttpDownloader {
    fn calculate_part_range(
        (part_size, parts_before_decrease): (u64, u64),
        index: u64,
    ) -> (u64, u64) {
        let mut start = index * part_size;
        let mut end = (index + 1) * part_size - 1;
        if index > parts_before_decrease {
            start -= index - parts_before_decrease;
        }
        if index >= parts_before_decrease {
            end -= index - parts_before_decrease + 1;
        }
        (start, end)
    }

    fn extract_part_range((start, end): (u64, u64)) -> String {
        format!("bytes={}-{}", start, end)
    }

    pub async fn start(&self) {
        let mut handles = vec![];
        let (sc, mut rc) = channel(100);
        let barrier = Arc::new(Barrier::new(self.config.threads_count as usize));
        let mut aggregators = vec![];

        for i in 0..self.config.threads_count {
            let (start, end) =
                HttpDownloader::calculate_part_range(self.config.split_result.unwrap(), i as u64);
            let part_range = HttpDownloader::extract_part_range((start, end));
            aggregators.push(BytesAggregator::new(start));

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
                    i as usize,
                )
                .await
            }));
        }

        let write_size = 1024 * 32;
        while let Some((chunk, index)) = rc.recv().await {
            self.info.add_to_downloaded_bytes(chunk.len() as u64);
            aggregators[index].push(chunk);
            if aggregators[index].len() >= write_size {
                let start_seek = aggregators[index].start_seek();
                let chunk = aggregators[index].merge_all();
            }
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
        sc: Sender<(Bytes, usize)>,
        barrier: Arc<Barrier>,
        index: usize,
    ) {
        let mut response = client
            .get(raw_url.as_str())
            .header(RANGE, part_range)
            .send()
            .await
            .unwrap();

        let mut download_strategy = DownloadStrategy::new(sc.clone(), throttle_config.task_speed());

        while let Some(chunk) = response.chunk().await.unwrap() {
            download_strategy.handle_chunk(chunk, &index).await;
            if throttle_config.has_throttle_changed() {
                download_strategy = DownloadStrategy::new(sc.clone(), throttle_config.task_speed());
                let wait_result = barrier.wait().await;
                if wait_result.is_leader() {
                    throttle_config.reset_has_throttle_changed();
                }
            }
        }
    }

    async fn process_chunk(sc: &mut Sender<(Bytes, usize)>, chunk: Bytes, index: &usize) {
        sc.send((chunk, *index)).await.unwrap();
    }
}

enum DownloadStrategy {
    NotThrottled {
        sc: Sender<(Bytes, usize)>,
    },
    Throttled {
        sc: Sender<(Bytes, usize)>,
        throttle: Throttler,
    },
}

impl DownloadStrategy {
    fn new(sc: Sender<(Bytes, usize)>, task_speed: u64) -> Self {
        if task_speed > 0 {
            let throttle = Throttler::new(task_speed);
            DownloadStrategy::Throttled { sc, throttle }
        } else {
            DownloadStrategy::NotThrottled { sc }
        }
    }

    async fn handle_chunk(&mut self, chunk: Bytes, index: &usize) {
        match self {
            DownloadStrategy::NotThrottled { sc } => {
                HttpDownloader::process_chunk(sc, chunk, index).await
            }
            DownloadStrategy::Throttled { sc, throttle } => {
                throttle.process_throttled(sc, chunk, index).await
            }
        }
    }
}
