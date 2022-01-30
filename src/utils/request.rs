use crate::error::{Error, ErrorResponseData, Result};

pub async fn send_request(request: reqwest::RequestBuilder) -> Result<reqwest::Response> {
    let response = request.send().await?;
    let status = response.status();

    if status.is_success() {
        return Ok(response);
    }

    let raw_response_body = response.text().await.ok();
    let errors: Option<ErrorResponseData> = if let Some(ref raw_response_body) = raw_response_body {
        serde_json::from_str(raw_response_body).ok()
    } else {
        None
    };

    return Err(Error::from_response_data(
        Some(status),
        errors,
        raw_response_body,
    ));
}