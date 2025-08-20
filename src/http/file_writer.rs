use std::{
    fs::{File, OpenOptions},
    io::{Seek, SeekFrom, Write},
};

use bytes::Bytes;

pub(super) struct FileWriter {
    file: File,
}

impl FileWriter {
    pub(super) fn open(filename: &str, is_new: bool) -> Result<Self, std::io::Error> {
        let file = if is_new {
            File::create(filename)?
        } else {
            OpenOptions::new().write(true).open(filename)?
        };

        Ok(Self { file })
    }

    pub(super) fn write_at(&mut self, offset: u64, buffer: Bytes) -> Result<(), std::io::Error> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&buffer)?;
        Ok(())
    }
}
