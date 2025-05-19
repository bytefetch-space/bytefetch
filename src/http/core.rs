use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use reqwest::{Client, header::RANGE};
use tokio::sync::Barrier;

use super::HttpDownloader;

const CHUNK_SIZE: usize = 16 * 1024;

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
        let throttle_speed = Some(32 * 1024);
        let throttle_timing =
            HttpDownloader::compute_throttle_timing(self.config.threads_count, throttle_speed);
        let throttle_timing_arc = Arc::new(throttle_timing);
        let throttle_changed = Arc::new(AtomicBool::new(true));
        let barrier = Arc::new(Barrier::new(self.config.threads_count as usize));

        for i in 0..self.config.threads_count {
            let part_range =
                HttpDownloader::extract_part_range(self.config.split_result.unwrap(), i as u64);
            let client = Arc::clone(&self.client);
            let raw_url = Arc::clone(&self.raw_url);
            let throttle_timing = Arc::clone(&throttle_timing_arc);
            let throttle_changed = Arc::clone(&throttle_changed);
            let barrier = Arc::clone(&barrier);
            handles.push(tokio::spawn(async move {
                HttpDownloader::download_part(
                    client,
                    raw_url,
                    part_range,
                    throttle_timing,
                    throttle_changed,
                    barrier,
                    i.into(),
                )
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
        throttle_timing: Arc<Option<(f32, f32)>>,
        throttle_changed: Arc<AtomicBool>,
        barrier: Arc<tokio::sync::Barrier>,
        index: f32,
    ) {
        let mut response = client
            .get(raw_url.as_str())
            .header(RANGE, part_range)
            .send()
            .await
            .unwrap();

        let (throttle_delay, throttle_sleep) = throttle_timing.unwrap_or_default();
        let is_throttled = true;

        loop {
            if throttle_changed.load(Ordering::Acquire) {
                let wait_result = barrier.wait().await;
                if wait_result.is_leader() {
                    throttle_changed.store(false, Ordering::Release);
                }
                tokio::time::sleep(Duration::from_secs_f32(throttle_delay * index)).await;
            }

            if let Some(chunk) = response.chunk().await.unwrap() {
            } else {
                break;
            }

            if is_throttled {
                tokio::time::sleep(Duration::from_secs_f32(throttle_sleep)).await;
            }
        }
    }

    fn compute_throttle_timing(
        threads_count: u8,
        throttle_speed: Option<u32>,
    ) -> Option<(f32, f32)> {
        let throttle_speed = throttle_speed?;
        // base_speed: 1 chunk per second per thread
        let base_speed = CHUNK_SIZE * threads_count as usize;
        let throttle_sleep = base_speed as f32 / throttle_speed as f32;
        let throttle_delay = throttle_sleep / threads_count as f32;
        Some((throttle_delay, throttle_sleep))
    }
}
