//! # ByteFetch
//!
//! `ByteFetch`  is a Rust library that makes HTTP file downloads easier to implement.
//!
//! 🚧  **This project is under active development.**
//!
//! The current version is an early release to reserve the crate name on [crates.io](https://crates.io/crates/bytefetch).
mod http;
pub use http::{HttpDownloader, HttpDownloaderSetupErrors, Status};
