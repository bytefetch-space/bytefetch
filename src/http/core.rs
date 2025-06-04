use std::{
    sync::{self, Arc},
    vec,
};

use bytes::Bytes;
use reqwest::{Client, header::RANGE};
use tokio::sync::{
    Barrier,
    mpsc::{Sender, channel},
};

use crate::http::progress_state::ProgressState;

use super::{
    HttpDownloader,
    bytes_aggregator::BytesAggregator,
    file_writer::FileWriter,
    throttle::{ThrottleConfig, Throttler},
};

type StdSender<T> = std::sync::mpsc::Sender<T>;
type StdReceiver<T> = std::sync::mpsc::Receiver<T>;

impl HttpDownloader {
    pub(super) fn calculate_part_range(
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
        let (sc, mut rc) = channel(512);
        let barrier = Arc::new(Barrier::new(self.config.threads_count as usize));
        let mut aggregators = vec![];
        let mut download_offsets = vec![];

        for i in 0..self.config.threads_count {
            let (start, end) =
                HttpDownloader::calculate_part_range(self.config.split_result.unwrap(), i as u64);
            let part_range = HttpDownloader::extract_part_range((start, end));
            aggregators.push(BytesAggregator::new(start));
            download_offsets.push(start);

            let client = Arc::clone(&self.client);
            let raw_url = Arc::clone(&self.raw_url);
            let sc_clone = sc.clone();
            let throttle_config = Arc::clone(&self.config.throttle_config);
            let barrier = Arc::clone(&barrier);
            tokio::spawn(async move {
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
            });
        }
        drop(sc);

        let (write_tx, write_rx) = sync::mpsc::channel();

        let filename = self.info.filename().to_string();
        let raw_url = (*self.raw_url).clone();
        let task_count = self.config.threads_count;
        let content_length = *self.info.content_length();
        let writer = move || {
            HttpDownloader::file_writer(
                write_rx,
                filename,
                raw_url,
                content_length,
                task_count,
                download_offsets,
            )
        };
        let writer_handle = tokio::task::spawn_blocking(writer);

        let write_size = 1024 * 32;
        while let Some((chunk, index)) = rc.recv().await {
            self.info.add_to_downloaded_bytes(chunk.len() as u64);
            aggregators[index].push(chunk);
            if aggregators[index].len() >= write_size {
                HttpDownloader::flush_to_writer(&write_tx, &mut aggregators[index], index);
            }
        }

        for index in 0..self.config.threads_count as usize {
            if aggregators[index].len() > 0 {
                HttpDownloader::flush_to_writer(&write_tx, &mut aggregators[index], index);
            }
        }

        drop(write_tx);
        writer_handle.await.unwrap();
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

    fn flush_to_writer(
        write_tx: &StdSender<(usize, u64, Bytes)>,
        aggregator: &mut BytesAggregator,
        index: usize,
    ) {
        let offset = aggregator.start_seek();
        let buffer = aggregator.merge_all();
        write_tx.send((index, offset, buffer)).unwrap();
    }

    fn file_writer(
        write_rx: StdReceiver<(usize, u64, Bytes)>,
        filename: String,
        url: String,
        content_length: Option<u64>,
        tasks_count: u8,
        download_offsets: Vec<u64>,
    ) {
        let mut file = FileWriter::new(&filename);
        let mut state =
            ProgressState::new(filename, url, content_length, tasks_count, download_offsets);
        while let Ok((index, offset, buffer)) = write_rx.recv() {
            let written_bytes = buffer.len() as u64;
            file.write_at(offset, buffer);
            state.update_progress(index, written_bytes);
        }
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
