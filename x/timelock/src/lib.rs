mod client;
mod config;
mod handler;
mod keeper;
mod message;
pub mod proto;
pub mod utils;

pub use client::*;
pub use config::*;
pub use handler::*;
pub use keeper::*;
pub use message::*;

// LOE Parameters from https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/info
// This is the "quicknet"
pub const LOE_URL: &str =
    "https://api.drand.sh/52db9ba70e0cc0f6eaf7803dd07447a1f5477735fd3f661792ba94600c84e971/";
const LOE_PUBLIC_KEY: &str = "83cf0f2896adee7eb8b5f01fcad3912212c437e0073e911fb90022d3e760183c8c4b450b6a0a6c3ac6a5776a2d1064510d1fec758c921cc22b0e17e63aaf4bcb5ed66304de9cf809bd274ca73bab4af5a6e9c76a4bc09e76eae8991ef5ece45a";
pub const LOE_GENESIS_TIME: u32 = 1692803367;
pub const LOE_PERIOD: u32 = 3;
const SECURITY_PARAM: usize = 10;
