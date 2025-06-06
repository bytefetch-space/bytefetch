use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
};

use bytes::Bytes;

pub(super) struct FileWriter {
    file: File,
}

impl FileWriter {
    fn new(filename: &str) -> Self {
        Self {
            file: File::create(filename).unwrap(),
        }
    }

    fn from(filename: &str) -> Self {
        Self {
            file: OpenOptions::new().write(true).open(filename).unwrap(),
        }
    }

    pub(super) fn open(filename: &str, is_new: bool) -> Self {
        if is_new {
            Self::new(filename)
        } else {
            Self::from(filename)
        }
    }

    pub(super) fn write_at(&mut self, offset: u64, buffer: Bytes) {
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.file.write_all(&buffer).unwrap();
    }
}
