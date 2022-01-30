//! Structures corresponding to the responses of the deta drive API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PutFile {
    pub name: String,
    pub project_id: String,
    pub drive_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct InitializeChunkedUpload {
    pub upload_id: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct EndChunkedUpload {
    pub name: String,
    pub upload_id: String,
    pub project_id: String,
    pub drive_name: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListFiles {
    pub paging: Option<ListFilesPaging>,
    pub names: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ListFilesPaging {
    pub size: usize,
    pub last: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DeleteFiles {
    pub deleted: Vec<String>,
    pub failed: Option<HashMap<String, String>>,
}
