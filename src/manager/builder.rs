use std::{collections::HashMap, io, marker::PhantomData, sync::Arc};

use parking_lot::Mutex;
use tokio::runtime::Runtime;

use crate::{
    DownloadManager,
    manager::{
        Callbacks,
        callbacks::{CanceledCallback, CompletedCallback, FailedCallback, ProgressCallback},
    },
};
use std::hash::Hash;

pub struct DownloadManagerBuilder<T> {
    pub(super) _marker: PhantomData<T>,
    pub(super) on_progress: Option<ProgressCallback<T>>,
    pub(super) on_completed: Option<CompletedCallback<T>>,
    pub(super) on_failed: Option<FailedCallback<T>>,
    pub(super) on_canceled: Option<CanceledCallback<T>>,
}

impl<T> DownloadManagerBuilder<T>
where
    T: Hash,
{
    pub fn build(self) -> io::Result<DownloadManager<T>> {
        Ok(DownloadManager {
            runtime: Arc::new(Runtime::new()?),
            callbacks: Arc::new(Callbacks {
                on_progress: self.on_progress,
                on_completed: self.on_completed,
                on_failed: self.on_failed,
                on_canceled: self.on_canceled,
            }),
            downloads: Mutex::new(HashMap::new()),
        })
    }
}
