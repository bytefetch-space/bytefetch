use std::{
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::{Duration, Instant},
};

use bytes::Bytes;
use tokio::{sync::mpsc::Sender, time::sleep};

pub struct Throttler {
    timestamp: Instant,
    bytes_downloaded: u64,
    target_speed: u64,
}

impl Throttler {
    pub(super) fn new(target_speed: u64) -> Self {
        Self {
            timestamp: Instant::now(),
            bytes_downloaded: 0,
            target_speed,
        }
    }

    pub(super) async fn process_throttled(
        &mut self,
        sc: &mut Sender<(Bytes, usize)>,
        chunk: Bytes,
        index: &usize,
    ) {
        self.bytes_downloaded += chunk.len() as u64;
        sc.send((chunk, *index)).await.unwrap();

        if self.bytes_downloaded < self.target_speed {
            return;
        }

        let sleep_time = self.compute_sleep_time();
        if sleep_time >= 0.001 {
            sleep(Duration::from_secs_f32(sleep_time)).await;
        }

        self.bytes_downloaded = 0;
        self.timestamp = Instant::now();
    }

    fn compute_sleep_time(&self) -> f32 {
        let diff_bytes = self.bytes_downloaded - self.target_speed;
        let additional_time;
        if diff_bytes > 0 {
            let bytes_rate = self.target_speed as f32 / diff_bytes as f32;
            additional_time = 1. / bytes_rate;
        } else {
            additional_time = 0.;
        }
        let elapsed_time = self.timestamp.elapsed().as_secs_f32();
        1. + additional_time - elapsed_time
    }
}

pub struct ThrottleConfig {
    has_throttle_changed: AtomicBool,
    task_speed: AtomicU64,
}

impl ThrottleConfig {
    pub(super) fn default() -> Self {
        Self {
            has_throttle_changed: AtomicBool::new(false),
            task_speed: AtomicU64::new(0),
        }
    }

    pub(super) fn set_task_speed(&self, task_speed: u64) {
        self.task_speed.store(task_speed, Ordering::Relaxed);
    }

    pub(super) fn has_throttle_changed(&self) -> bool {
        self.has_throttle_changed.load(Ordering::Relaxed)
    }

    pub(super) fn reset_has_throttle_changed(&self) {
        self.has_throttle_changed.store(false, Ordering::Relaxed);
    }

    pub(super) fn task_speed(&self) -> u64 {
        self.task_speed.load(Ordering::Relaxed)
    }

    pub(super) fn change_throttle_speed(&self, throttle_speed: Option<u64>, tasks_count: u64) {
        let task_speed = throttle_speed.unwrap_or_default() / tasks_count;
        self.task_speed.store(task_speed, Ordering::Relaxed);
        self.has_throttle_changed.store(true, Ordering::Relaxed);
    }
}
