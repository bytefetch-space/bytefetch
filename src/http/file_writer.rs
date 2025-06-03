use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
};

use bytes::Bytes;

pub(super) struct FileWriter {
    file: File,
}

impl FileWriter {
    pub(super) fn new(filename: &str) -> Self {
        Self {
            file: File::create(filename).unwrap(),
        }
    }

    pub(super) fn write_at(&mut self, offset: u64, buffer: Bytes) {
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.write_all(&buffer).unwrap();
    }
}
