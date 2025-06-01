use std::io::SeekFrom;

use bytes::Bytes;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};

pub(super) struct FileWriter {
    file: File,
}

impl FileWriter {
    pub(super) async fn new(filename: &str) -> Self {
        Self {
            file: File::create(filename).await.unwrap(),
        }
    }

    pub(super) async fn write_at(&mut self, offset: u64, buffer: Bytes) {
        self.file.seek(SeekFrom::Start(offset)).await.unwrap();
        self.file.write_all(&buffer).await.unwrap();
    }
}
