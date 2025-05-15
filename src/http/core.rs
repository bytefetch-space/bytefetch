use std::sync::Arc;

use reqwest::{Client, header::RANGE};

use super::HttpDownloader;

impl HttpDownloader {
    fn extract_part_range((part_size, parts_before_decrease): (u64, u64), index: u64) -> String {
        let mut start = index * part_size;
        let mut end = (index + 1) * part_size - 1;
        if index > parts_before_decrease {
            start -= index - parts_before_decrease;
        }
        if index >= parts_before_decrease {
            end -= index - parts_before_decrease + 1;
        }
        format!("bytes={}-{}", start, end)
    }

    pub async fn start(&self) {
        let mut handles = vec![];

        for i in 0..self.config.threads_count {
            let part_range =
                HttpDownloader::extract_part_range(self.config.split_result.unwrap(), i as u64);
            let client = Arc::clone(&self.client);
            let raw_url = Arc::clone(&self.raw_url);
            handles.push(tokio::spawn(async {
                HttpDownloader::download_part(client, raw_url, part_range).await
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }
    }

    async fn download_part(client: Arc<Client>, raw_url: Arc<String>, part_range: String) {
        let mut response = client
            .get(raw_url.as_str())
            .header(RANGE, part_range)
            .send()
            .await
            .unwrap();

        while let Some(chunk) = response.chunk().await.unwrap() {}
    }
}
