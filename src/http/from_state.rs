use reqwest::Client;
use std::{marker::PhantomData, sync::Arc};

use crate::{
    HttpDownloader,
    http::{
        DownloadHandle, Error, HttpDownloadConfig, HttpDownloadMode, ProgressState, builder_utils,
        info::HttpDownloadInfo, options::DownloadOptions,
    },
};

pub struct ClientRequired;
pub struct FromStateBuilder;

pub struct HttpDownloaderFromStateBuilder<State = FromStateBuilder> {
    filename: String,
    client: Option<Client>,
    state: PhantomData<State>,
    pub(super) options: DownloadOptions,
}

impl HttpDownloaderFromStateBuilder<ClientRequired> {
    pub fn client(mut self, client: Client) -> HttpDownloaderFromStateBuilder<FromStateBuilder> {
        self.client = Some(client);
        HttpDownloaderFromStateBuilder {
            client: self.client,
            state: PhantomData::<FromStateBuilder>,
            filename: self.filename,
            options: self.options,
        }
    }
}

impl HttpDownloaderFromStateBuilder {
    pub(super) fn new(filename: String) -> HttpDownloaderFromStateBuilder<ClientRequired> {
        HttpDownloaderFromStateBuilder::<ClientRequired> {
            client: None,
            state: PhantomData::<ClientRequired>,
            filename,
            options: DownloadOptions::default(),
        }
    }

    fn generate_info(
        filename: String,
        content_length: Option<u64>,
        tasks_count: u8,
    ) -> HttpDownloadInfo {
        HttpDownloadInfo::default()
            .set_filename(filename)
            .set_content_length(content_length)
            .set_is_resumable(tasks_count > 0)
    }

    fn get_byte_ranges_and_downloaded_bytes(
        config: &HttpDownloadConfig,
        mode: &HttpDownloadMode,
        state: &ProgressState,
    ) -> (Vec<(u64, u64)>, u64) {
        match mode {
            HttpDownloadMode::NonResumable => (vec![], 0),
            HttpDownloadMode::ResumableStream => {
                let progress = state.get_progress(0);
                (vec![(progress, 0)], progress)
            }
            HttpDownloadMode::ResumableMultithread => {
                let mut byte_ranges = vec![];
                let mut downloaded_bytes = 0;
                for index in 0..config.tasks_count as usize {
                    let (start, end) = builder_utils::calculate_part_range(
                        config.split_result.unwrap(),
                        index as u64,
                    );
                    let start_offset = state.get_progress(index);
                    byte_ranges.push((start_offset, end));
                    downloaded_bytes += start_offset - start;
                }
                (byte_ranges, downloaded_bytes)
            }
        }
    }

    pub fn build(self) -> Result<HttpDownloader, Error> {
        let mut config = HttpDownloadConfig::default()
            .try_set_directory(self.options.directory)?
            .set_timeout(self.options.timeout)
            .mark_resumed();

        let mut url = String::new();
        let mut content_length = None;
        let mut tasks_count = 0;

        let state = ProgressState::load(
            config.directory.join(&self.filename),
            &mut url,
            &mut content_length,
            &mut tasks_count,
        )?;
        config.tasks_count = tasks_count;
        config.set_throttle_speed(self.options.throttle_speed);

        let info = Self::generate_info(self.filename, content_length, tasks_count);
        let mode = builder_utils::determine_mode(tasks_count, &info);
        config.split_result = builder_utils::try_split_content(&mode, &content_length, tasks_count);

        let (byte_ranges, number) =
            Self::get_byte_ranges_and_downloaded_bytes(&config, &mode, &state);

        info.add_to_downloaded_bytes(number);

        Ok(HttpDownloader {
            client: Arc::new(self.client.unwrap()),
            raw_url: Arc::new(url),
            info,
            mode,
            config,
            byte_ranges,
            handle: Arc::new(DownloadHandle::new(self.options.token)),
        })
    }
}
