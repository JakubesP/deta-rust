use crate::error::Result;
use crate::utils::send_request;
use serde_json::json;

pub async fn put_file_request(
    base_url: &str,
    x_api_key: &str,
    file_name: &str,
    data: Vec<u8>,
    content_type: Option<&str>,
) -> Result<reqwest::Response> {
    let mut request = reqwest::Client::new()
        .post(format!("{}/files", base_url))
        .query(&[("name", file_name)])
        .body(data)
        .header("X-Api-Key", x_api_key);

    if let Some(content_type) = content_type {
        request = request.header("Content-Type", content_type);
    }

    send_request(request).await
}

pub async fn get_file_request(
    base_url: &str,
    x_api_key: &str,
    file_name: &str,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .get(format!("{}/files/download", base_url))
        .query(&[("name", file_name)])
        .header("X-Api-Key", x_api_key);

    send_request(request).await
}

pub async fn list_files_request(
    base_url: &str,
    x_api_key: &str,
    limit: Option<u32>,
    prefix: Option<&str>,
    last_name: Option<&str>,
) -> Result<reqwest::Response> {
    let mut request = reqwest::Client::new()
        .get(format!("{}/files", base_url))
        .header("X-Api-Key", x_api_key);

    let mut query_params: Vec<(&'static str, String)> = vec![];
    if let Some(limit) = limit {
        query_params.push(("limit", format!("{}", limit)));
    }
    if let Some(prefix) = prefix {
        query_params.push(("prefix", prefix.into()));
    }
    if let Some(last_name) = last_name {
        query_params.push(("last", last_name.into()));
    }

    request = request.query(&query_params);

    send_request(request).await
}

pub async fn delete_files_request(
    base_url: &str,
    x_api_key: &str,
    names: &[String],
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .delete(format!("{}/files", base_url))
        .header("X-Api-Key", x_api_key)
        .json(&json!({ "names": names }));
    send_request(request).await
}

pub async fn initialize_chunked_upload_request(
    base_url: &str,
    x_api_key: &str,
    name: &str,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .post(format!("{}/uploads", base_url))
        .query(&[("name", name)])
        .header("X-Api-Key", x_api_key);

    send_request(request).await
}

pub async fn upload_chunk_request(
    base_url: &str,
    x_api_key: &str,
    name: &str,
    upload_id: &str,
    part: usize,
    data: bytes::Bytes,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .post(format!("{}/uploads/{}/parts", base_url, upload_id))
        .query(&[("name", name), ("part", &part.to_string())])
        .header("X-Api-Key", x_api_key)
        .body(data);
    send_request(request).await
}

pub async fn abort_chunked_upload_request(
    base_url: &str,
    x_api_key: &str,
    name: &str,
    upload_id: &str,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .delete(format!("{}/uploads/{}", base_url, upload_id))
        .query(&[("name", name)])
        .header("X-Api-Key", x_api_key);

    send_request(request).await
}

pub async fn end_chunked_upload_request(
    base_url: &str,
    x_api_key: &str,
    name: &str,
    upload_id: &str,
) -> Result<reqwest::Response> {
    let request = reqwest::Client::new()
        .patch(format!("{}/uploads/{}", base_url, upload_id))
        .query(&[("name", name)])
        .header("X-Api-Key", x_api_key);
    send_request(request).await
}
