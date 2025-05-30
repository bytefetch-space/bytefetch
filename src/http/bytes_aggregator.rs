use bytes::{Bytes, BytesMut};
use std::collections::VecDeque;

pub(super) struct BytesAggregator {
    queue: VecDeque<Bytes>,
    total_len: usize,
    start_seek: u64,
}

impl BytesAggregator {
    pub(super) fn new(start: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            total_len: 0,
            start_seek: start,
        }
    }

    pub(super) fn push(&mut self, chunk: Bytes) {
        self.total_len += chunk.len();
        self.queue.push_back(chunk);
    }

    pub(super) fn merge_all(&mut self) -> Bytes {
        let mut buf = BytesMut::with_capacity(self.total_len);
        while let Some(chunk) = self.queue.pop_front() {
            buf.extend_from_slice(&chunk);
        }
        self.start_seek += self.total_len as u64;
        self.total_len = 0;
        buf.freeze()
    }

    pub(super) fn len(&self) -> usize {
        self.total_len
    }

    pub(super) fn start_seek(&self) -> u64 {
        self.start_seek
    }
}
