use std::{
    sync::{self, Arc},
    vec,
};

use bytes::Bytes;
use reqwest::Response;
use tokio::sync::{
    Barrier,
    mpsc::{Sender, channel},
};

use crate::http::{
    progress_state::ProgressState,
    request_utils::{RequestBuilderExt, basic_request},
};

use super::{
    HttpDownloader,
    bytes_aggregator::BytesAggregator,
    file_writer::FileWriter,
    throttle::{ThrottleConfig, Throttler},
};

type StdSender<T> = std::sync::mpsc::Sender<T>;
type StdReceiver<T> = std::sync::mpsc::Receiver<T>;

impl HttpDownloader {
    fn extract_part_range((start, end): (u64, u64)) -> String {
        format!("bytes={}-{}", start, end)
    }

    fn extract_start_range(start: u64) -> String {
        format!("bytes={}-", start)
    }

    pub async fn start(&self) {
        let (sc, mut rc) = channel(512);
        let barrier = Arc::new(Barrier::new(self.config.tasks_count as usize));
        let mut aggregators = vec![];
        let mut download_offsets = vec![];

        for index in 0..self.config.tasks_count as usize {
            let (start, end) = self.byte_ranges[index];
            let part_range = HttpDownloader::extract_part_range((start, end));
            aggregators.push(BytesAggregator::new(start));
            download_offsets.push(start);
            let request = basic_request(&self.client, &self.raw_url).with_range(part_range);
            let response = request.send().await.unwrap();
            self.spawn_download_task(response, &sc, &barrier, index);
        }
        drop(sc);

        let (write_tx, write_rx) = sync::mpsc::channel();

        let file = FileWriter::open(self.info.filename(), self.config.is_new);
        let state = ProgressState::new(
            self.info.filename(),
            (*self.raw_url).clone(),
            *self.info.content_length(),
            self.config.tasks_count,
            download_offsets,
        );
        let writer = move || HttpDownloader::file_writer(write_rx, file, state);
        let writer_handle = tokio::task::spawn_blocking(writer);

        let write_size = 1024 * 32;
        while let Some((chunk, index)) = rc.recv().await {
            self.info.add_to_downloaded_bytes(chunk.len() as u64);
            aggregators[index].push(chunk);
            if aggregators[index].len() >= write_size {
                HttpDownloader::flush_to_writer(&write_tx, &mut aggregators[index], index);
            }
        }

        for index in 0..self.config.tasks_count as usize {
            if aggregators[index].len() > 0 {
                HttpDownloader::flush_to_writer(&write_tx, &mut aggregators[index], index);
            }
        }

        drop(write_tx);
        writer_handle.await.unwrap();
    }

    fn spawn_download_task(
        &self,
        response: Response,
        sc: &Sender<(Bytes, usize)>,
        barrier: &Arc<Barrier>,
        index: usize,
    ) {
        let throttle_config = Arc::clone(&self.config.throttle_config);
        let sc_clone = sc.clone();
        let barrier = Arc::clone(barrier);
        tokio::spawn(async move {
            HttpDownloader::download(response, throttle_config, sc_clone, barrier, index).await
        });
    }

    async fn download(
        mut response: Response,
        throttle_config: Arc<ThrottleConfig>,
        sc: Sender<(Bytes, usize)>,
        barrier: Arc<Barrier>,
        index: usize,
    ) {
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
        mut file: FileWriter,
        mut state: ProgressState,
    ) {
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
