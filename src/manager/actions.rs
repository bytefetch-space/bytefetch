use super::DownloadManager;
use std::{hash::Hash, sync::Arc};

impl<T> DownloadManager<T>
where
    T: Hash + Eq + Clone + Send + 'static,
{
    pub fn add_download(&self, key: T, url: &str) {
        self.urls.lock().insert(key, Arc::new(String::from(url)));
    }
}
