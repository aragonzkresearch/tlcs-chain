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
const LOE_PUBLIC_KEY: &str = "a0b862a7527fee3a731bcb59280ab6abd62d5c0b6ea03dc4ddf6612fdfc9d01f01c31542541771903475eb1ec6615f8d0df0b8b6dce385811d6dcf8cbefb8759e5e616a3dfd054c928940766d9a5b9db91e3b697e5d70a975181e007f87fca5e";
pub const LOE_GENESIS_TIME: u32 = 1692803367;
pub const LOE_PERIOD: u32 = 3;
const SECURITY_PARAM: usize = 10;
