use std::{collections::HashMap, marker::PhantomData};

use parking_lot::Mutex;

use crate::DownloadManager;
use std::hash::Hash;

pub struct DownloadManagerBuilder<T> {
    pub(super) _marker: PhantomData<T>,
}

impl<T> DownloadManagerBuilder<T>
where
    T: Hash,
{
    pub fn build(&self) -> DownloadManager<T> {
        DownloadManager {
            urls: Mutex::new(HashMap::new()),
        }
    }
}
