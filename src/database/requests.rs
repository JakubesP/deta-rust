use crate::error::Result;
use crate::utils::send_request;
use serde::Serialize;
use serde_json::json;
use super::ItemUpdates;

pub async fn put_items_request<T>(
    base_url: &str,
    x_api_key: &str,
    items: &[T],
) -> Result<reqwest::Response>
where
    T: Serialize,
{
    let request = reqwest::Client::new()
        .put(format!("{}/items", base_url))
        .header("X-Api-Key", x_api_key)
        .json(&json!({ "items": &items }));

    send_request(request).await
}

pub async fn get_item_request(
    base_url: &str,
    x_api_key: &str,
    key: &str,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .get(format!("{}/items/{}", base_url, key))
        .header("X-Api-Key", x_api_key);

    send_request(request).await
}

pub async fn delete_item_request(
    base_url: &str,
    x_api_key: &str,
    key: &str,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .delete(format!("{}/items/{}", base_url, key))
        .header("X-Api-Key", x_api_key);

    send_request(request).await
}

pub async fn insert_item_request<T>(
    base_url: &str,
    x_api_key: &str,
    item: &T,
) -> Result<reqwest::Response>
where
    T: Serialize,
{
    let request = reqwest::Client::new()
        .post(format!("{}/items", base_url))
        .header("X-Api-Key", x_api_key)
        .json(&json!({ "item": item }));

    send_request(request).await
}

pub async fn query_items_request(
    base_url: &str,
    x_api_key: &str,
    limit: Option<u32>,
    last: Option<&str>,
    query: Option<&[serde_json::Value]>,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .post(format!("{}/query", base_url))
        .header("X-Api-Key", x_api_key)
        .json(&json!({
            "limit": limit,
            "last": last,
            "query": query
        }));

    send_request(request).await
}

pub async fn update_item_request(
    base_url: &str,
    x_api_key: &str,
    key: &str,
    updates: &ItemUpdates,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .patch(format!("{}/items/{}", base_url, key))
        .header("X-Api-Key", x_api_key)
        .json(updates);

    send_request(request).await
}
