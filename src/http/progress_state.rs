use std::{
    fs::File,
    io::{Seek, SeekFrom, Write},
};

const U64_SIZE: u64 = 8;
const STATE_EXTENSION: &str = ".bfstate";

pub(super) struct ProgressState {
    file: File,
    progress_offset: u64,
    segment_offsets: Vec<u64>,
}

impl ProgressState {
    pub(super) fn new(
        filename: String,
        url: String,
        tasks_count: u8,
        download_offsets: Vec<u64>,
    ) -> Self {
        let mut file = File::create(filename + STATE_EXTENSION).unwrap();

        let url_len = url.len() as u32;
        file.write_all(&url_len.to_le_bytes()).unwrap();
        file.write_all(url.as_bytes()).unwrap();
        file.write_all(&tasks_count.to_le_bytes()).unwrap();

        for offset in &download_offsets {
            file.write_all(&offset.to_le_bytes()).unwrap();
        }

        let progress_offset = 4 + url_len as u64 + 1;

        Self {
            file,
            progress_offset,
            segment_offsets: download_offsets,
        }
    }

    pub(super) fn update_progress(&mut self, index: usize, written_bytes: u64) {
        let offset = self.progress_offset + index as u64 * U64_SIZE;
        self.file.seek(SeekFrom::Start(offset)).unwrap();
        self.segment_offsets[index] += written_bytes;
        self.file
            .write_all(&self.segment_offsets[index].to_le_bytes())
            .unwrap();
        self.file.flush().unwrap();
    }
}
