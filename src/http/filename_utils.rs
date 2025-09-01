use std::{path::Path, sync::LazyLock};

use percent_encoding::percent_decode_str;
use regex::Regex;
use reqwest::header::HeaderValue;

static HEADER_FILENAME_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"filename\*?=(?:UTF-8''|)?(?:"((?:[^"\\]|\\.)*)"|([^;\s]+))"#).unwrap()
});
static URL_FILENAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"/([^/?#;]+)/?(?:[?#;].*)?$").unwrap());

pub(super) fn extract_filename_from_header(
    content_disposition: &Option<&HeaderValue>,
) -> Option<String> {
    content_disposition
        .and_then(|value| value.to_str().ok())
        .and_then(|v| HEADER_FILENAME_REGEX.captures(v))
        .and_then(|captures| {
            captures
                .get(1)
                .or(captures.get(2))
                .map(|s| s.as_str().to_owned())
        })
}

pub(super) fn extract_filename_from_url(raw_url: &str) -> Option<String> {
    let captures = URL_FILENAME_REGEX.captures(raw_url);
    captures.and_then(|captures| captures.get(1).map(|m| m.as_str().to_string()))
}

pub(super) fn percent_decode(input: &str) -> String {
    percent_decode_str(input).decode_utf8_lossy().to_string()
}

fn is_html_type(content_type: &Option<&HeaderValue>) -> bool {
    content_type
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(';').next())
        .map(str::trim)
        == Some("text/html")
}

fn split_filename(filename: Option<String>) -> (Option<String>, Option<String>) {
    match filename {
        Some(name) => {
            let path = Path::new(&name);
            let stem = path.file_stem().and_then(|s| s.to_str()).map(String::from);
            let ext = path.extension().and_then(|e| e.to_str()).map(String::from);
            (stem, ext)
        }
        None => (None, None),
    }
}

pub(super) fn extract_filename(
    raw_url: &str,
    content_disposition: &Option<&HeaderValue>,
    content_type: &Option<&HeaderValue>,
) -> String {
    let (h_stem, h_ext) = split_filename(extract_filename_from_header(content_disposition));
    let (u_stem, u_ext) = split_filename(extract_filename_from_url(raw_url));

    let stem = h_stem.or(u_stem).unwrap_or(String::from("download"));
    let ext = h_ext.or(u_ext).unwrap_or_default();

    let mut raw_filename = if ext.is_empty() {
        stem
    } else {
        format!("{}.{}", stem, ext)
    };

    if is_html_type(content_type) {
        raw_filename.push_str(".html");
    }

    percent_decode(&raw_filename)
}
