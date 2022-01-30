//! Structures corresponding to the responses of the deta drive API.

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Items<T> {
    pub items: Vec<T>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PutItems<T> {
    pub processed: Items<T>,
    pub failed: Option<Items<T>>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteItem {
    pub key: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchItems<T> {
    pub paging: FetchItemsPaging,
    pub items: Vec<T>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FetchItemsPaging {
    pub size: usize,
    pub last: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct UpdateItem {
    pub key: String,
    pub set: Option<serde_json::Value>,
    pub increment: Option<serde_json::Value>,
    pub append: Option<serde_json::Value>,
    pub prepend: Option<serde_json::Value>,
    pub delete: Option<serde_json::Value>
}
