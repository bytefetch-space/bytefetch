mod actions;
mod builder;

use builder::DownloadManagerBuilder;
use parking_lot::Mutex;

use std::{collections::HashMap, hash::Hash, marker::PhantomData, sync::Arc};

pub struct DownloadManager<T>
where
    T: Hash,
{
    urls: Mutex<HashMap<T, Arc<String>>>,
}

impl<T> DownloadManager<T>
where
    T: Hash,
{
    pub fn builder() -> DownloadManagerBuilder<T> {
        DownloadManagerBuilder {
            _marker: PhantomData,
        }
    }
}
