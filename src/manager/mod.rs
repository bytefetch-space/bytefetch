mod actions;
mod builder;
mod callbacks;

use builder::DownloadManagerBuilder;
use parking_lot::Mutex;
use tokio::runtime::Runtime;
use tokio_util::sync::CancellationToken;

use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};

use crate::manager::callbacks::Callbacks;

pub struct DownloadManager<T>
where
    T: Hash,
{
    runtime: Arc<Runtime>,
    urls: Mutex<HashMap<T, Arc<String>>>,
    callbacks: Arc<Callbacks<T>>,
    tokens: Mutex<HashMap<T, CancellationToken>>,
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
