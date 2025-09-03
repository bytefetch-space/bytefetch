mod actions;
mod builder;
mod callbacks;
pub(crate) mod config;
pub(crate) mod entry;

use builder::DownloadManagerBuilder;
use parking_lot::Mutex;
use tokio::runtime::Runtime;

use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};

use crate::manager::{callbacks::Callbacks, entry::DownloadEntry};

pub struct DownloadManager<T>
where
    T: Hash,
{
    runtime: Arc<Runtime>,
    callbacks: Arc<Callbacks<T>>,
    downloads: Mutex<HashMap<T, Arc<DownloadEntry>>>,
}

impl<T> DownloadManager<T>
where
    T: Hash,
{
    pub fn builder() -> DownloadManagerBuilder<T> {
        DownloadManagerBuilder {
            _marker: PhantomData,
            on_progress: None,
            on_completed: None,
            on_failed: None,
            on_canceled: None,
        }
    }
}
