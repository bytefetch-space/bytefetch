use reqwest::{RequestBuilder, header::RANGE};

pub(super) trait RequestBuilderExt {
    fn with_range(self, range: String) -> Self;
}

impl RequestBuilderExt for RequestBuilder {
    fn with_range(self, part_range: String) -> Self {
        self.header(RANGE, part_range)
    }
}
