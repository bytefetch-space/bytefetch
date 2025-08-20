use std::sync::Arc;

use tokio::sync::Barrier;

use crate::http::bytes_aggregator::BytesAggregator;

pub(super) struct HttpDownloadSession {
    pub(super) aggregators: Vec<BytesAggregator>,
    pub(super) barrier: Arc<Barrier>,
}

impl HttpDownloadSession {
    pub(super) fn new(tasks_count: usize) -> Self {
        Self {
            aggregators: vec![],
            barrier: Arc::new(Barrier::new(tasks_count)),
        }
    }
}
