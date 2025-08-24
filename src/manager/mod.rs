mod actions;
mod builder;
mod callbacks;

use builder::DownloadManagerBuilder;
use parking_lot::Mutex;

use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};

use crate::manager::callbacks::Callbacks;

pub struct DownloadManager<T>
where
    T: Hash,
{
    urls: Mutex<HashMap<T, Arc<String>>>,
    callbacks: Arc<Callbacks<T>>,
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
