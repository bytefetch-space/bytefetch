use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

const U64_SIZE: u64 = 8;
const STATE_EXTENSION: &str = ".bfstate";

pub trait FromLeBytes<const N: usize>: Sized {
    fn from_le_bytes(bytes: [u8; N]) -> Self;
}

macro_rules! impl_from_le_bytes {
    ($t:ty) => {
        impl FromLeBytes<{ std::mem::size_of::<$t>() }> for $t {
            fn from_le_bytes(bytes: [u8; std::mem::size_of::<$t>()]) -> Self {
                <$t>::from_le_bytes(bytes)
            }
        }
    };
}

impl_from_le_bytes!(u8);
impl_from_le_bytes!(u32);
impl_from_le_bytes!(u64);

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

    pub(super) fn load(filename: String, url: &mut String) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(filename + STATE_EXTENSION)
            .unwrap();

        let url_len: u32 = ProgressState::read_le_int(&mut file);
        let mut url_bytes = vec![0u8; url_len as usize];
        file.read_exact(&mut url_bytes).unwrap();
        *url = String::from_utf8(url_bytes).unwrap();

        let tasks_count: u8 = ProgressState::read_le_int(&mut file);
        let mut segment_offsets = Vec::with_capacity(tasks_count as usize);

        for _ in 0..tasks_count {
            let offset: u64 = ProgressState::read_le_int(&mut file);
            segment_offsets.push(offset);
        }

        let progress_offset = 4 + url_len as u64 + 1;

        Self {
            file,
            progress_offset,
            segment_offsets,
        }
    }

    fn read_le_int<T: FromLeBytes<N>, const N: usize>(file: &mut File) -> T {
        let mut bytes = [0u8; N];
        file.read_exact(&mut bytes).unwrap();
        T::from_le_bytes(bytes)
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
