use percent_encoding::percent_decode_str;
use regex::Regex;
use reqwest::header::HeaderValue;

pub(super) fn extract_filename_from_header(
    content_disposition: &Option<&HeaderValue>,
) -> Option<String> {
    let filename_regex = Regex::new(r#"filename\*?=(?:UTF-8''|)?"?([^";\r\n]+)"?"#).unwrap();
    content_disposition
        .and_then(|value| filename_regex.captures(value.to_str().unwrap()))
        .and_then(|captures| Some(captures[1].to_string()))
}

pub(super) fn extract_filename_from_url(raw_url: &str) -> Option<String> {
    let filename_regex = Regex::new(r"/([^/?#;]+)/?(?:[?#;].*)?$").unwrap();
    let captures = filename_regex.captures(raw_url);
    captures.and_then(|captures| Some(captures[1].to_string()))
}

pub(super) fn percent_decode(input: &str) -> String {
    percent_decode_str(input).decode_utf8_lossy().to_string()
}

pub(super) fn extract_filename(
    raw_url: &str,
    content_disposition: &Option<&HeaderValue>,
) -> String {
    let raw_filename = extract_filename_from_header(content_disposition)
        .or_else(|| extract_filename_from_url(raw_url))
        .unwrap_or_else(|| "download".into());
    percent_decode(&raw_filename)
}
