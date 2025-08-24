use crate::{http::Error, manager::builder::DownloadManagerBuilder};

use std::hash::Hash;

impl<T> DownloadManagerBuilder<T>
where
    T: Hash,
{
    pub fn on_progress<F>(mut self, cb: F) -> Self
    where
        F: Fn(T, u64, u64) + Send + Sync + 'static,
    {
        self.on_progress = Some(Box::new(cb));
        self
    }

    pub fn on_complete<F>(mut self, cb: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.on_completed = Some(Box::new(cb));
        self
    }

    pub fn on_failed<F>(mut self, cb: F) -> Self
    where
        F: Fn(T, Error) + Send + Sync + 'static,
    {
        self.on_failed = Some(Box::new(cb));
        self
    }

    pub fn on_canceled<F>(mut self, cb: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.on_canceled = Some(Box::new(cb));
        self
    }
}

pub(super) type ProgressCallback<T> = Box<dyn Fn(T, u64, u64) + Send + Sync>;
pub(super) type CompletedCallback<T> = Box<dyn Fn(T) + Send + Sync>;
pub(super) type FailedCallback<T> = Box<dyn Fn(T, Error) + Send + Sync>;
pub(super) type CanceledCallback<T> = Box<dyn Fn(T) + Send + Sync>;

pub(super) struct Callbacks<T> {
    pub(super) on_progress: Option<ProgressCallback<T>>,
    pub(super) on_completed: Option<CompletedCallback<T>>,
    pub(super) on_failed: Option<FailedCallback<T>>,
    pub(super) on_canceled: Option<CanceledCallback<T>>,
}
