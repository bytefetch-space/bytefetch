use crate::http::{HttpDownloadMode, info::HttpDownloadInfo};

fn split_content(content_length: u64, thread_number: u64) -> (u64, u64) {
    let mut remainder = content_length % thread_number;
    let mut part_size = content_length / thread_number;
    if remainder > 0 {
        part_size += 1
    } else {
        remainder = thread_number
    }
    (part_size, remainder) // Example: split_content(1003, 4) returns (251, 3), meaning 3 parts are 251 bytes and 1 part is 250 bytes
}

pub(super) fn try_split_content(
    mode: &HttpDownloadMode,
    content_length: &Option<u64>,
    threads_count: u8,
) -> Option<(u64, u64)> {
    if *mode == HttpDownloadMode::NonResumable || *mode == HttpDownloadMode::ResumableStream {
        return None;
    }
    Some(split_content(content_length.unwrap(), threads_count as u64))
}

pub(super) fn determine_mode(threads_count: u8, info: &HttpDownloadInfo) -> HttpDownloadMode {
    match (threads_count, info.content_length(), info.is_resumable()) {
        (_, _, false) => return HttpDownloadMode::NonResumable,
        (_, None, true) | (1, _, true) => return HttpDownloadMode::ResumableStream,
        (_, _, true) => return HttpDownloadMode::ResumableMultithread,
    }
}
