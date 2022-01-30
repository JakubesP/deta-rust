use crate::error::{Error, Result};
use serde::de::DeserializeOwned;

pub async fn parse_response_body<T>(response: reqwest::Response) -> Result<T>
where
    T: DeserializeOwned,
{
    let raw_response_body = response.text().await.ok();
    parse_raw_response_text(raw_response_body).await
}

async fn parse_raw_response_text<T>(raw_response_text: Option<String>) -> Result<T>
where
    T: DeserializeOwned,
{
    if raw_response_text.is_none() {
        return Err(Error::from_failed_deserialization(None));
    }

    let raw_response_text = raw_response_text.unwrap();

    let model = serde_json::from_str::<T>(&raw_response_text);

    if let Err(_) = model {
        return Err(Error::from_failed_deserialization(Some(raw_response_text)));
    }

    Ok(model.unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, PartialEq, Eq, Debug)]
    struct SampleModel {
        data: i32,
    }

    #[tokio::test]
    pub async fn parse_raw_response_text_for_valid_data() {
        let text = r#"{ "data": 10 }"#;
        let result = parse_raw_response_text::<SampleModel>(Some(text.into())).await;
        assert!(matches!(result, Ok(_)));
        let model = result.unwrap();
        assert_eq!(model, SampleModel { data: 10 });
    }

    #[tokio::test]
    pub async fn parse_raw_response_text_for_none() {
        let result = parse_raw_response_text::<SampleModel>(None).await;
        assert!(matches!(result, Err(_)));
        let error = result.err().unwrap();
        assert!(error.is_body_deserialization());
        assert_eq!(error.get_raw_response_data(), None);
    }

    #[tokio::test]
    pub async fn parse_raw_response_text_for_incompatible_model() {
        let text = r#"{ "data": "text data" }"#;
        let result = parse_raw_response_text::<SampleModel>(Some(text.into())).await;
        assert!(matches!(result, Err(_)));
        let error = result.err().unwrap();
        assert!(error.is_body_deserialization());
        assert_eq!(error.get_raw_response_data(), Some(text.into()));
    }

    #[tokio::test]
    pub async fn parse_raw_response_text_for_invalid_json() {
        let text = r#"{ "data"; }"#;
        let result = parse_raw_response_text::<SampleModel>(Some(text.into())).await;
        assert!(matches!(result, Err(_)));
        let error = result.err().unwrap();
        assert!(error.is_body_deserialization());
        assert_eq!(error.get_raw_response_data(), Some(text.into()));
    }
}
