use std::sync::Arc;

use tokio::sync::Barrier;

use crate::http::bytes_aggregator::BytesAggregator;

pub(super) struct HttpDownloadSession {
    pub(super) aggregators: Vec<BytesAggregator>,
    pub(super) download_offsets: Vec<u64>,
    pub(super) barrier: Arc<Barrier>,
}

impl HttpDownloadSession {
    pub(super) fn new(tasks_count: usize) -> Self {
        Self {
            aggregators: vec![],
            download_offsets: vec![],
            barrier: Arc::new(Barrier::new(tasks_count)),
        }
    }

    pub(super) fn take_download_offsets(&mut self) -> Vec<u64> {
        std::mem::take(&mut self.download_offsets)
    }
}
