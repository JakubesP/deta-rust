//! Deta-base service SDK.
//! Check [deta docs](https://docs.deta.sh/docs/base/http) for more information.

use crate::constants;
use crate::deta_client::DetaClient;
use crate::error::Result;
use crate::utils;
use serde::de::DeserializeOwned;
use serde::Serialize;
mod common;
pub mod models;
pub mod query;
mod requests;
pub mod updates;

/// Stores the necessary information and methods to
/// work with the [deta-base](https://docs.deta.sh/docs/base/http) api.
pub struct Database {
    base_url: String,
    x_api_key: String,
}

impl Database {
    /// Creates an `Database` instance.
    pub fn new(client: &DetaClient, database_name: &str) -> Self {
        let base_url = format!(
            "{}/{}/{}",
            constants::DATABASE_API_URL,
            client.project_id(),
            database_name
        );

        let x_api_key = client.api_key().to_owned();

        Self {
            base_url,
            x_api_key,
        }
    }

    /// Creates or overwrites collections of elements
    /// depending on whether a element with a given key already exists in the database or not.
    pub async fn put_items<T>(&self, items: &[T]) -> Result<models::PutItems<T>>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = requests::put_items_request(&self.base_url, &self.x_api_key, items).await?;
        utils::parse_response_body(response).await
    }

    /// Returns an item with a given key.
    pub async fn get_item<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let response_result =
            requests::get_item_request(&self.base_url, &self.x_api_key, key).await;

        if let Err(ref error) = response_result {
            if error.is_not_found() {
                return Ok(None);
            }
        }

        let response = response_result?;
        utils::parse_response_body(response).await
    }

    /// Deletes an item with a given key.
    pub async fn delete_item(&self, key: &str) -> Result<models::DeleteItem> {
        let response = requests::delete_item_request(&self.base_url, &self.x_api_key, key).await?;
        utils::parse_response_body(response).await
    }

    /// Adds a new item. If the specified object contains a key that already exists in the database,
    /// the operation fails (collision error).
    pub async fn insert_item<T>(&self, item: &T) -> Result<T>
    where
        T: DeserializeOwned + Serialize,
    {
        let response = requests::insert_item_request(&self.base_url, &self.x_api_key, item).await?;
        utils::parse_response_body(response).await
    }

    /// Fetch items for database.
    /// The `query` value is described by the [`Query`](query::Query) type.
    /// Check [deta docs](https://docs.deta.sh/docs/base/sdk/#queries) for more information.
    pub async fn fetch_items<T>(
        &self,
        limit: Option<u32>,
        last: Option<&str>,
        query: Option<query::Query>,
    ) -> Result<models::FetchItems<T>>
    where
        T: DeserializeOwned,
    {
        let query_value;
        if let Some(query) = query {
            query_value = Some(query.render()?);
        } else {
            query_value = None;
        }

        let response = requests::query_items_request(
            &self.base_url,
            &self.x_api_key,
            limit,
            last,
            query_value,
        )
        .await?;
        utils::parse_response_body(response).await
    }

    /// Updates an item with the specified key.
    /// The updates are described by the [`Updates`](updates::Updates) type.
    /// Check [deta docs](https://docs.deta.sh/docs/base/sdk/#update-operations) for more information.
    pub async fn update_item(
        &self,
        key: &str,
        updates: updates::Updates,
    ) -> Result<models::UpdateItem> {
        let response_result =
            requests::update_item_request(&self.base_url, &self.x_api_key, key, updates.render()?)
                .await;

        let response = response_result?;
        utils::parse_response_body(response).await
    }
}
