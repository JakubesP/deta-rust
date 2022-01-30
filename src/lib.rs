//! The [Deta-Rust](https://github.com/JakubesP/deta-rust) is a simple unofficial [Deta](https://www.deta.sh/) SDK for Rust lang.
//! 
//! You can see [examples](https://github.com/JakubesP/deta-rust) to get you started more quickly.
//!
//! Have fun 😀

mod constants;
pub mod database;
mod deta_client;
pub mod drive;
pub mod error;
mod utils;
pub use deta_client::DetaClient;

// Re-exports
pub use serde;
pub use serde_json;
