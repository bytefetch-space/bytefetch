//! # ByteFetch
//!
//! `ByteFetch`  is a Rust library that makes HTTP file downloads easier to implement.
//!
//! 🚧  **This project is under active development.**
//!
//! # 💡 Tips
//!
//! **⚠️ Avoid Native TLS Memory Leaks**
//!
//! To prevent potential memory leaks, configure `reqwest` to use the **rustls-tls** backend.
//! This issue originates from native TLS behavior, not from this crate itself.
//!
//! Example `Cargo.toml` configuration:
//!
//! ```toml
//! reqwest = { version = "VERSION", default-features = false, features = ["rustls-tls"] }
//! ```
//! You can observe the memory behavior yourself using tools like [`heaptrack`](https://github.com/KDE/heaptrack) or similar memory profiling tools.
mod http;
pub use http::{HttpDownloader, HttpDownloaderSetupErrors, Status};
