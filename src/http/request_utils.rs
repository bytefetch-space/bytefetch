use std::{sync::Arc, time::Duration};

use crate::http::Error;
use reqwest::{Client, RequestBuilder, Response, header::RANGE};

pub(super) trait RequestBuilderExt {
    fn with_range(self, range: String) -> Self;
    async fn send_with_timeout(self) -> Result<Response, Error>;
}

impl RequestBuilderExt for RequestBuilder {
    fn with_range(self, part_range: String) -> Self {
        self.header(RANGE, part_range)
    }

    async fn send_with_timeout(self) -> Result<Response, Error> {
        let result = tokio::time::timeout(Duration::from_secs(5), self.send()).await;
        match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(e)) => Err(Error::Network(e)),
            Err(_) => Err(Error::Timeout),
        }
    }
}

pub(super) fn basic_request(client: &Arc<Client>, url: &str) -> RequestBuilder {
    client.get(url)
}
