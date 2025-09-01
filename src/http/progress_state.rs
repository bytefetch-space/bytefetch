use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::PathBuf,
};

const U64_SIZE: u64 = 8;
const STATE_EXTENSION: &str = ".bfstate";

type Result<T> = std::result::Result<T, std::io::Error>;

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
        filename: PathBuf,
        url: String,
        content_length: Option<u64>,
        tasks_count: u8,
        download_offsets: Vec<u64>,
    ) -> Result<Self> {
        let state_path = format!("{}{}", filename.display(), STATE_EXTENSION);
        let mut file = File::create(state_path)?;

        let url_serialized_size = ProgressState::write_string(&mut file, url)?; // 4 + N Bytes
        let content_length_serialized_size =
            ProgressState::write_option_u64(&mut file, content_length)?; // 1(None) or 9(Value) Bytes 

        ProgressState::write_le_int(&mut file, tasks_count)?; // 1 Byte
        for offset in &download_offsets {
            ProgressState::write_le_int(&mut file, *offset)?; // 8 Bytes
        }

        let progress_offset = url_serialized_size + content_length_serialized_size + 1;

        Ok(Self {
            file,
            progress_offset,
            segment_offsets: download_offsets,
        })
    }

    pub(super) fn load(
        filename: PathBuf,
        url: &mut String,
        content_length: &mut Option<u64>,
        tasks_count: &mut u8,
    ) -> Result<Self> {
        let state_path = format!("{}{}", filename.display(), STATE_EXTENSION);
        let mut file = OpenOptions::new().read(true).write(true).open(state_path)?;

        let (url_serialized_size, deserialized_url) = ProgressState::read_string(&mut file)?;
        *url = deserialized_url;
        let (content_length_serialized_size, deserialized_content_length) =
            ProgressState::read_option_u64(&mut file)?;
        *content_length = deserialized_content_length;

        *tasks_count = ProgressState::read_le_int(&mut file)?;
        let mut segment_offsets = Vec::with_capacity(*tasks_count as usize);

        for _ in 0..*tasks_count {
            let offset: u64 = ProgressState::read_le_int(&mut file)?;
            segment_offsets.push(offset);
        }

        let progress_offset = url_serialized_size + content_length_serialized_size + 1;

        Ok(Self {
            file,
            progress_offset,
            segment_offsets,
        })
    }

    fn write_le_int<T: LeBytes<N>, const N: usize>(file: &mut File, val: T) -> Result<()> {
        file.write_all(&val.to_le_bytes())
    }

    fn read_le_int<T: LeBytes<N>, const N: usize>(file: &mut File) -> Result<T> {
        let mut bytes = [0u8; N];
        file.read_exact(&mut bytes)?;
        Ok(T::from_le_bytes(bytes))
    }

    fn write_string(file: &mut File, str: String) -> Result<u64> {
        let len = str.len() as u32;
        ProgressState::write_le_int(file, len)?;
        file.write_all(str.as_bytes())?;
        Ok(4 + len as u64)
    }

    fn read_string(file: &mut File) -> Result<(u64, String)> {
        let url_len: u32 = ProgressState::read_le_int(file)?;
        let mut url_bytes = vec![0u8; url_len as usize];
        file.read_exact(&mut url_bytes)?;
        let url = String::from_utf8(url_bytes)
            .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;
        Ok((4 + url.len() as u64, url))
    }

    fn write_option_u64(file: &mut File, val: Option<u64>) -> Result<u64> {
        match val {
            Some(v) => {
                file.write_all(&[1])?;
                ProgressState::write_le_int(file, v)?;
                Ok(9)
            }
            None => {
                file.write_all(&[0])?;
                Ok(1)
            }
        }
    }

    fn read_option_u64(file: &mut File) -> Result<(u64, Option<u64>)> {
        let mut flag = [0u8; 1];
        file.read_exact(&mut flag)?;

        match flag[0] {
            1 => Ok((9, Some(ProgressState::read_le_int(file)?))),
            _ => Ok((1, None)),
        }
    }

    pub(super) fn get_progress(&self, index: usize) -> u64 {
        self.segment_offsets[index]
    }
}

pub(super) trait ProgressUpdater {
    fn update_progress(&mut self, index: usize, written_bytes: u64) -> Result<()>;
}

impl ProgressUpdater for ProgressState {
    fn update_progress(&mut self, index: usize, written_bytes: u64) -> Result<()> {
        let offset = self.progress_offset + index as u64 * U64_SIZE;
        self.file.seek(SeekFrom::Start(offset))?;
        self.segment_offsets[index] += written_bytes;
        self.file
            .write_all(&self.segment_offsets[index].to_le_bytes())?;
        self.file.flush()?;
        Ok(())
    }
}

pub(super) struct NoOpProgressState;

impl ProgressUpdater for NoOpProgressState {
    #[inline(always)]
    // no-op
    fn update_progress(&mut self, _index: usize, _written_bytes: u64) -> Result<()> {
        Ok(())
    }
}
