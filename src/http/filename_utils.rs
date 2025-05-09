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

pub(super) fn extract_filename(
    raw_url: &str,
    content_disposition: &Option<&HeaderValue>,
) -> String {
    let mut filename_option = extract_filename_from_header(content_disposition);
    if let Some(filename) = filename_option {
        return filename;
    };
    filename_option = extract_filename_from_url(raw_url);
    filename_option.unwrap_or(String::from("download"))
}
