use std::{
    sync::{self, Arc},
    time::Duration,
};

use bytes::Bytes;
use reqwest::RequestBuilder;
use tokio::{
    select,
    sync::{
        Barrier,
        mpsc::{Sender, channel},
    },
    task::JoinHandle,
    time::{Instant, sleep},
};
use tokio_util::sync::CancellationToken;

use crate::http::{
    DownloadHandle, Error, HttpDownloadMode,
    progress_state::{NoOpProgressState, ProgressState, ProgressUpdater},
    request_utils::{RequestBuilderExt, basic_request},
    session::HttpDownloadSession,
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
        self.handle.mark_downloading();
        let (download_tx, mut download_rx) = channel(512);
        let mut session = HttpDownloadSession::new(self.config.tasks_count as usize);

        match self.mode {
            HttpDownloadMode::NonResumable => {
                self.spawn_nonresumable_download_task(&mut session, download_tx)
                    .await
            }
            HttpDownloadMode::ResumableStream => {
                self.spawn_resumable_download_task(&mut session, download_tx)
                    .await
            }
            HttpDownloadMode::ResumableMultithread => {
                self.spawn_multiple_download_tasks(&mut session, download_tx)
                    .await
            }
        }

        let (write_tx, write_rx) = sync::mpsc::channel();

        let file = FileWriter::open(self.info.filename(), self.config.is_new);
        let state = ProgressState::new(
            self.info.filename(),
            (*self.raw_url).clone(),
            self.info.content_length(),
            self.config.tasks_count,
            session.take_download_offsets(),
        );

        let writer_handle = self.spawn_writer(write_rx, file, state);

        let write_size = 1024 * 32;
        while let Some((chunk, index)) = download_rx.recv().await {
            self.info.add_to_downloaded_bytes(chunk.len() as u64);
            session.aggregators[index].push(chunk);
            if session.aggregators[index].len() >= write_size {
                HttpDownloader::flush_to_writer(&write_tx, &mut session.aggregators[index], index);
            }
        }

        for index in 0..session.aggregators.len() {
            if session.aggregators[index].len() > 0 {
                HttpDownloader::flush_to_writer(&write_tx, &mut session.aggregators[index], index);
            }
        }

        drop(write_tx);
        writer_handle.await.unwrap();
        self.handle.mark_completed();
    }

    async fn spawn_nonresumable_download_task(
        &self,
        session: &mut HttpDownloadSession,
        download_tx: Sender<(Bytes, usize)>,
    ) {
        session.aggregators.push(BytesAggregator::new(0));

        let request = basic_request(&self.client, &self.raw_url);

        self.spawn_download_task(request, &download_tx, &session.barrier, 0);
    }

    async fn spawn_resumable_download_task(
        &self,
        session: &mut HttpDownloadSession,
        download_tx: Sender<(Bytes, usize)>,
    ) {
        self.spawn_download_for_range(session, &download_tx, (self.byte_ranges[0].0, None), 0)
            .await;
    }

    async fn spawn_multiple_download_tasks(
        &self,
        session: &mut HttpDownloadSession,
        download_tx: Sender<(Bytes, usize)>,
    ) {
        for index in 0..self.config.tasks_count as usize {
            let (start, end) = self.byte_ranges[index];
            self.spawn_download_for_range(session, &download_tx, (start, Some(end)), index)
                .await;
        }
    }

    async fn spawn_download_for_range(
        &self,
        session: &mut HttpDownloadSession,
        download_tx: &Sender<(Bytes, usize)>,
        (start, end): (u64, Option<u64>),
        index: usize,
    ) {
        let part_range = match end {
            Some(end) => Self::extract_part_range((start, end)),
            None => Self::extract_start_range(start),
        };

        session.aggregators.push(BytesAggregator::new(start));
        session.download_offsets.push(start);

        let request = basic_request(&self.client, &self.raw_url).with_range(part_range);

        self.spawn_download_task(request, download_tx, &session.barrier, index);
    }

    fn spawn_download_task(
        &self,
        request: RequestBuilder,
        download_tx: &Sender<(Bytes, usize)>,
        barrier: &Arc<Barrier>,
        index: usize,
    ) {
        let throttle_config = Arc::clone(&self.config.throttle_config);
        let download_tx = download_tx.clone();
        let barrier = Arc::clone(barrier);
        let handle = Arc::clone(&self.handle);
        let timeout = self.config.timeout;
        tokio::spawn(async move {
            HttpDownloader::download(
                request,
                throttle_config,
                download_tx,
                barrier,
                index,
                handle,
                timeout,
            )
            .await
        });
    }

    async fn download(
        request: RequestBuilder,
        throttle_config: Arc<ThrottleConfig>,
        download_tx: Sender<(Bytes, usize)>,
        barrier: Arc<Barrier>,
        index: usize,
        handle: Arc<DownloadHandle>,
        timeout: Duration,
    ) {
        let mut response = match request.send_with_timeout(timeout).await {
            Ok(response) => response,
            Err(e) => {
                handle.mark_failed(e);
                return;
            }
        };

        let mut download_strategy = DownloadStrategy::new(
            download_tx.clone(),
            handle.token.clone(),
            throttle_config.task_speed(),
        );

        let sleep_fut = sleep(timeout);
        tokio::pin!(sleep_fut);

        loop {
            sleep_fut.as_mut().reset(Instant::now() + timeout);

            select! {
                _ = handle.token.cancelled() => {
                    handle.mark_canceled();
                    break;
                }

                chunk_res = response.chunk() => {
                    match chunk_res {
                        Ok(Some(chunk)) => {
                            download_strategy.handle_chunk(chunk, &index).await;
                            if throttle_config.has_throttle_changed() {
                                download_strategy =
                                    DownloadStrategy::new(download_tx.clone(), handle.token.clone(), throttle_config.task_speed());
                                let wait_result = barrier.wait().await;
                                if wait_result.is_leader() {
                                    throttle_config.reset_has_throttle_changed();
                                }
                            }
                        }
                        Ok(None) => break,
                        Err(e) => {
                            handle.mark_failed(e);
                            break;
                        }
                    }
                }

                _ = sleep_fut.as_mut() => {
                    handle.mark_failed(Error::Timeout);
                    break;
                }
            }
        }
    }

    fn spawn_writer(
        &self,
        write_rx: StdReceiver<(usize, u64, Bytes)>,
        file: FileWriter,
        state: ProgressState,
    ) -> JoinHandle<()> {
        if self.mode == HttpDownloadMode::NonResumable {
            let writer = move || HttpDownloader::file_writer(write_rx, file, NoOpProgressState);
            tokio::task::spawn_blocking(writer)
        } else {
            let writer = move || HttpDownloader::file_writer(write_rx, file, state);
            tokio::task::spawn_blocking(writer)
        }
    }

    async fn process_chunk(download_tx: &mut Sender<(Bytes, usize)>, chunk: Bytes, index: &usize) {
        download_tx.send((chunk, *index)).await.unwrap();
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

    fn file_writer<U: ProgressUpdater>(
        write_rx: StdReceiver<(usize, u64, Bytes)>,
        mut file: FileWriter,
        mut state: U,
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
        download_tx: Sender<(Bytes, usize)>,
    },
    Throttled {
        download_tx: Sender<(Bytes, usize)>,
        throttle: Throttler,
        token: CancellationToken,
    },
}

impl DownloadStrategy {
    fn new(download_tx: Sender<(Bytes, usize)>, token: CancellationToken, task_speed: u64) -> Self {
        if task_speed > 0 {
            let throttle = Throttler::new(task_speed);
            DownloadStrategy::Throttled {
                download_tx,
                throttle,
                token,
            }
        } else {
            DownloadStrategy::NotThrottled { download_tx }
        }
    }

    async fn handle_chunk(&mut self, chunk: Bytes, index: &usize) {
        match self {
            DownloadStrategy::NotThrottled { download_tx } => {
                HttpDownloader::process_chunk(download_tx, chunk, index).await
            }
            DownloadStrategy::Throttled {
                download_tx,
                throttle,
                token,
            } => {
                throttle
                    .process_throttled(download_tx, token, chunk, index)
                    .await
            }
        }
    }
}
