//! # compress-lz77-rs
//!
//! A pure-Rust LZ77 sliding window compression library.
//!
//! # Modules
//!
//! - [`window`] — Sliding window buffer for back-reference management.
//! - [`match_`] — Match finding with configurable search parameters.
//! - [`token`] — Length-distance token representation.
//! - [`encode`] — LZ77 encoding with optional lazy matching.
//! - [`decode`] — LZ77 decoding from token streams.
//!
//! # Quick Start
//!
//! ```
//! use compress_lz77_rs::{encode, decode};
//!
//! let data = b"abracadabra abracadabra";
//! let tokens = encode::encode(data, 4096, 3, 258);
//! let decoded = decode::decode(&tokens);
//! assert_eq!(data.as_slice(), decoded.as_slice());
//! ```

pub mod window;
pub mod match_;
pub mod token;
pub mod encode;
pub mod decode;

pub use token::Token;

#[cfg(test)]
mod tests;
