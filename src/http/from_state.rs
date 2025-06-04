use reqwest::Client;
use std::{marker::PhantomData, sync::Arc};

use crate::{
    HttpDownloader,
    http::{HttpDownloadConfig, ProgressState, builder_utils, info::HttpDownloadInfo},
};

pub struct ClientRequired;
pub struct FromStateBuilder;

pub struct HttpDownloaderFromStateBuilder<State = FromStateBuilder> {
    filename: String,
    client: Option<Client>,
    state: PhantomData<State>,
}

impl HttpDownloaderFromStateBuilder<ClientRequired> {
    pub fn client(mut self, client: Client) -> HttpDownloaderFromStateBuilder<FromStateBuilder> {
        self.client = Some(client);
        HttpDownloaderFromStateBuilder {
            client: self.client,
            state: PhantomData::<FromStateBuilder>,
            filename: self.filename,
        }
    }
}

impl HttpDownloaderFromStateBuilder {
    pub(super) fn new(filename: String) -> HttpDownloaderFromStateBuilder<ClientRequired> {
        HttpDownloaderFromStateBuilder::<ClientRequired> {
            client: None,
            state: PhantomData::<ClientRequired>,
            filename,
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

    pub fn build(self) -> HttpDownloader {
        let mut url = String::new();
        let mut content_length = None;
        let mut tasks_count = 0;
        let state = ProgressState::load(
            &self.filename,
            &mut url,
            &mut content_length,
            &mut tasks_count,
        );
        let info = HttpDownloaderFromStateBuilder::generate_info(
            self.filename,
            content_length,
            tasks_count,
        );
        let mode = builder_utils::determine_mode(tasks_count, &info);
        let mut config = HttpDownloadConfig::default()
            .set_thread_count(Some(tasks_count))
            .unwrap();
        config.split_result = builder_utils::try_split_content(&mode, &content_length, tasks_count);

        let mut downloaded = 0;
        for i in 0..tasks_count as usize {
            let start =
                HttpDownloader::calculate_part_range(config.split_result.unwrap(), i as u64).0;
            println!(
                "{:?}",
                HttpDownloader::calculate_part_range(config.split_result.unwrap(), i as u64)
            );
            downloaded += state.get_progress(i) - start;
        }
        info.add_to_downloaded_bytes(downloaded);

        HttpDownloader {
            client: Arc::new(self.client.unwrap()),
            raw_url: Arc::new(url),
            info,
            mode,
            config,
        }
    }
}
