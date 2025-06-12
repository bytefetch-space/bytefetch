use crate::http::filename_utils;
use reqwest::header::HeaderValue;

#[test]
fn test_extract_filename_from_header() {
    let content_disposition_value = HeaderValue::from_static("attachment; filename=example.txt");
    let content_disposition = Some(&content_disposition_value);
    let result = filename_utils::extract_filename_from_header(&content_disposition);
    assert_eq!(result, Some(String::from("example.txt")).to_owned());
}

#[test]
fn test_extract_filename_from_url() {
    let url = "https://test.com/test.mp4";
    let result = filename_utils::extract_filename_from_url(&url);
    assert_eq!(result, Some(String::from("test.mp4")));
}

#[test]
fn test_percent_decode() {
    let url = "100%25_complete.mp3";
    let result = filename_utils::percent_decode(&url);
    assert_eq!(result, String::from("100%_complete.mp3"));
}
