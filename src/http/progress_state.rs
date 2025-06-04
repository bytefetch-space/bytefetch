use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
};

const U64_SIZE: u64 = 8;
const STATE_EXTENSION: &str = ".bfstate";

trait LeBytes<const N: usize>: Sized {
    fn from_le_bytes(bytes: [u8; N]) -> Self;
    fn to_le_bytes(&self) -> [u8; N];
}

macro_rules! impl_le_bytes {
    ($($t:ty),*) => {
        $(
            impl LeBytes<{ std::mem::size_of::<$t>() }> for $t {
                fn from_le_bytes(bytes: [u8; std::mem::size_of::<$t>()]) -> Self {
                    <$t>::from_le_bytes(bytes)
                }

                fn to_le_bytes(&self) -> [u8; std::mem::size_of::<$t>()] {
                    <$t>::to_le_bytes(*self)
                }
            }
        )*
    };
}

impl_le_bytes!(u8, u32, u64);

#[derive(Debug)]
pub(super) struct ProgressState {
    file: File,
    progress_offset: u64,
    segment_offsets: Vec<u64>,
}

impl ProgressState {
    pub(super) fn new(
        filename: String,
        url: String,
        content_length: Option<u64>,
        tasks_count: u8,
        download_offsets: Vec<u64>,
    ) -> Self {
        let mut file = File::create(filename + STATE_EXTENSION).unwrap();

        let url_serialized_size = ProgressState::write_string(&mut file, url); // 4 + N Bytes
        let content_length_serialized_size =
            ProgressState::write_option_u64(&mut file, content_length); // 1(None) or 9(Value) Bytes 

        ProgressState::write_le_int(&mut file, tasks_count); // 1 Byte
        for offset in &download_offsets {
            ProgressState::write_le_int(&mut file, *offset); // 8 Bytes
        }

        let progress_offset = url_serialized_size + content_length_serialized_size + 1;

        Self {
            file,
            progress_offset,
            segment_offsets: download_offsets,
        }
    }

    pub(super) fn load(
        filename: &str,
        url: &mut String,
        content_length: &mut Option<u64>,
        tasks_count: &mut u8,
    ) -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(filename.to_string() + STATE_EXTENSION)
            .unwrap();

        let (url_serialized_size, deserialized_url) = ProgressState::read_string(&mut file);
        *url = deserialized_url;
        let (content_length_serialized_size, deserialized_content_length) =
            ProgressState::read_option_u64(&mut file);
        *content_length = deserialized_content_length;

        *tasks_count = ProgressState::read_le_int(&mut file);
        let mut segment_offsets = Vec::with_capacity(*tasks_count as usize);

        for _ in 0..*tasks_count {
            let offset: u64 = ProgressState::read_le_int(&mut file);
            segment_offsets.push(offset);
        }

        let progress_offset = url_serialized_size + content_length_serialized_size + 1;

        Self {
            file,
            progress_offset,
            segment_offsets,
        }
    }

    fn write_le_int<T: LeBytes<N>, const N: usize>(file: &mut File, val: T) {
        file.write_all(&val.to_le_bytes()).unwrap();
    }

    fn read_le_int<T: LeBytes<N>, const N: usize>(file: &mut File) -> T {
        let mut bytes = [0u8; N];
        file.read_exact(&mut bytes).unwrap();
        T::from_le_bytes(bytes)
    }

    fn write_string(file: &mut File, str: String) -> u64 {
        let len = str.len() as u32;
        ProgressState::write_le_int(file, len);
        file.write_all(str.as_bytes()).unwrap();
        4 + len as u64
    }

    fn read_string(file: &mut File) -> (u64, String) {
        let url_len: u32 = ProgressState::read_le_int(file);
        let mut url_bytes = vec![0u8; url_len as usize];
        file.read_exact(&mut url_bytes).unwrap();
        let url = String::from_utf8(url_bytes).unwrap();
        (4 + url.len() as u64, url)
    }

    fn write_option_u64(file: &mut File, val: Option<u64>) -> u64 {
        match val {
            Some(v) => {
                file.write_all(&[1]).unwrap();
                ProgressState::write_le_int(file, v);
                return 9;
            }
            None => {
                file.write_all(&[0]).unwrap();
                return 1;
            }
        }
    }

    fn read_option_u64(file: &mut File) -> (u64, Option<u64>) {
        let mut flag = [0u8; 1];
        file.read_exact(&mut flag).unwrap();

        match flag[0] {
            1 => {
                return (9, Some(ProgressState::read_le_int(file)));
            }
            _ => return (1, None),
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

    pub(super) fn get_progress(&self, index: usize) -> u64 {
        self.segment_offsets[index]
    }
}
