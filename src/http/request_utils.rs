use std::sync::Arc;

use reqwest::{Client, RequestBuilder, header::RANGE};

pub(super) trait RequestBuilderExt {
    fn with_range(self, range: String) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
    fn with_range(self, part_range: String) -> Self {
        self.header(RANGE, part_range)
    }
}

pub(super) fn basic_request(client: &Arc<Client>, url: &str) -> RequestBuilder {
    client.get(url)
}
