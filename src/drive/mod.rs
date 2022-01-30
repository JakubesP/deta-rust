//! Deta-drive service SDK.
//! Check [deta docs](https://docs.deta.sh/docs/drive/http) for more information.

use crate::deta_client::DetaClient;
pub mod models;
mod requests;
use crate::constants;
use crate::error::Result;
use crate::utils;

/// Stores the necessary information and methods to
/// work with the [deta-drive](https://docs.deta.sh/docs/drive/http) API.
pub struct Drive {
    base_url: String,
    x_api_key: String,
}

impl Drive {
    /// Creates an `Drive` instance.
    pub fn new(client: &DetaClient, drive_name: &str) -> Self {
        let base_url = format!(
            "{}/{}/{}",
            constants::DRIVE_API_URL,
            client.project_id(),
            drive_name
        );

        let x_api_key = client.api_key().to_owned();

        Self {
            base_url,
            x_api_key,
        }
    }

    async fn get_chunked_upload_object(
        &self,
        name: &str,
    ) -> Result<models::InitializeChunkedUpload> {
        let response =
            requests::initialize_chunked_upload_request(&self.base_url, &self.x_api_key, name)
                .await?;
        Ok(utils::parse_response_body(response).await?)
    }

    async fn perform_chunked_upload(
        &self,
        name: &str,
        data: Vec<u8>,
    ) -> Result<models::EndChunkedUpload> {
        let bytes: bytes::Bytes = data.into();
        let upload_id = self.get_chunked_upload_object(name).await?.upload_id;
        let content_length = bytes.len();
        let chunk_size = constants::MAX_DATA_CHUNK_SIZE;
        let mut part = 1;

        for idx in (0..content_length).step_by(chunk_size) {
            let end = content_length.min(idx + chunk_size);
            let chunk = bytes.slice(idx..end);
            let upload_result = requests::upload_chunk_request(
                &self.base_url,
                &self.x_api_key,
                name,
                &upload_id,
                part,
                chunk,
            )
            .await;
            if let Err(error) = upload_result {
                requests::abort_chunked_upload_request(
                    &self.base_url,
                    &self.x_api_key,
                    name,
                    &upload_id,
                )
                .await?;
                return Err(error);
            }
            part += 1;
        }

        let response =
            requests::end_chunked_upload_request(&self.base_url, &self.x_api_key, name, &upload_id)
                .await?;
        Ok(utils::parse_response_body(response).await?)
    }

    /// Uploads the file to the server.
    /// If the amount of data to be uploaded exceeds 10MB, chunked uploading will be used.
    pub async fn put_file(
        &self,
        name: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> Result<PutFileResult> {
        if data.len() <= constants::MAX_DATA_CHUNK_SIZE {
            let response = requests::put_file_request(
                &self.base_url,
                &self.x_api_key,
                name,
                data,
                content_type,
            )
            .await?;
            return Ok(PutFileResult::SinglePut(
                utils::parse_response_body(response).await?,
            ));
        }

        Ok(PutFileResult::ChunkedUpload(
            self.perform_chunked_upload(name, data).await?,
        ))
    }

    /// Returns a raw data as type [bytes::Bytes](bytes::Bytes).
    pub async fn get_file_as_buffer(&self, name: &str) -> Result<Option<bytes::Bytes>> {
        let response_result =
            requests::get_file_request(&self.base_url, &self.x_api_key, name).await;

        if let Err(ref error) = response_result {
            if error.is_not_found() {
                return Ok(None);
            }
        }

        let response = response_result?;
        let bytes = response.bytes().await?;
        Ok(Some(bytes))
    }

    /// Returns a raw data as type Vec<u8>.
    pub async fn get_file_as_u8_vec(&self, name: &str) -> Result<Option<Vec<u8>>> {
        let bytes = self.get_file_as_buffer(name).await?;
        if bytes.is_none() {
            return Ok(None);
        }
        let bytes = bytes.unwrap();
        Ok(Some(bytes.to_vec()))
    }

    /// Lists file names.
    pub async fn list_files(
        &self,
        limit: Option<u32>,
        prefix: Option<&str>,
        last_name: Option<&str>,
    ) -> Result<models::ListFiles> {
        let response =
            requests::list_files_request(&self.base_url, &self.x_api_key, limit, prefix, last_name)
                .await?;
        return Ok(utils::parse_response_body(response).await?);
    }

    /// Deletes files by the names specified in the slice.
    pub async fn delete_files(&self, names: &[String]) -> Result<models::DeleteFiles> {
        let response =
            requests::delete_files_request(&self.base_url, &self.x_api_key, names).await?;
        return Ok(utils::parse_response_body(response).await?);
    }
}

/// Positive response variants to file upload.

#[derive(Debug, Clone)]
pub enum PutFileResult {
    /// File size less than or equal to 10MB.
    SinglePut(models::PutFile),
    /// File size greater than 10MB.
    ChunkedUpload(models::EndChunkedUpload),
}
