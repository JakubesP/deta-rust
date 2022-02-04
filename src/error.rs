//! When performing an action in the deta drive API or deserializing the response fails.

use serde::{Deserialize, Serialize};
use std::error::Error as StdError;
use thiserror::Error as ThisError;

pub type Result<T> = std::result::Result<T, Error>;
pub(crate) type BoxError = Box<dyn StdError + Send + Sync>;

/// A type representing all possible failures that may occur during integration with deta.
#[derive(ThisError, Debug)]
pub struct Error {
    kind: Kind,
    source: Option<BoxError>,
    raw_response_data: Option<String>,
}

impl Error {
    pub(crate) fn from_response_data(
        status: Option<reqwest::StatusCode>,
        errors: Option<ErrorResponseData>,
        raw_response_data: Option<String>,
    ) -> Self {
        Self {
            kind: Kind::ResponseStatus(ResponseStatusKind::from_code(status), errors),
            source: None,
            raw_response_data,
        }
    }

    pub(crate) fn from_failed_deserialization(raw_response_data: Option<String>) -> Self {
        Self {
            kind: Kind::DataDeserialization,
            source: None,
            raw_response_data,
        }
    }

    /// Checks whether the error is caused by any unsuccessful response status.
    pub fn is_response(&self) -> bool {
        matches!(self.kind, Kind::ResponseStatus(_, _))
    }

    /// Checks whether the error is caused by the 404 response status.
    pub fn is_not_found(&self) -> bool {
        matches!(
            self.kind,
            Kind::ResponseStatus(ResponseStatusKind::NotFound, _)
        )
    }

    /// Checks whether the error is caused by the 400 response status.
    pub fn is_bad_request(&self) -> bool {
        matches!(
            self.kind,
            Kind::ResponseStatus(ResponseStatusKind::BadRequest, _)
        )
    }

    /// Case if the error is due to deserialization of the response for **successful** completion of the task.
    /// The failure to deserialise the response for an incorrect status will never result in this error.
    pub fn is_body_deserialization(&self) -> bool {
        matches!(self.kind, Kind::DataDeserialization)
    }

    /// Returns a reference to the [`Kind`](Kind) enum.
    pub fn get_kind(&self) -> &Kind {
        &self.kind
    }

    /// Returns raw deta's response body, if exists.
    pub fn get_raw_response_data(&self) -> Option<&str> {
        self.raw_response_data.as_deref()
    }
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        let kind = if error.is_body() {
            Kind::Other("Request or response body error".into())
        } else if error.is_builder() {
            Kind::Other("Request builder error".into())
        } else if error.is_connect() {
            Kind::Connection("Connection error".into())
        } else if error.is_decode() {
            Kind::DataDeserialization
        } else if error.is_redirect() {
            Kind::Connection("Error following redirect".into())
        } else if error.is_request() {
            Kind::Other("Error sending request".into())
        } else if error.is_timeout() {
            Kind::Connection("Timeout exceeded".into())
        } else if error.is_status() {
            Kind::ResponseStatus(ResponseStatusKind::from_code(error.status()), None)
        } else {
            Kind::Other("Unknown error".into())
        };

        Self {
            kind,
            source: Some(error.into()),
            raw_response_data: None,
        }
    }
}

impl std::convert::From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Self {
            kind: Kind::DataDeserialization,
            source: Some(error.into()),
            raw_response_data: None,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            Kind::Connection(msg) => {
                f.write_str(&format!("Connection exception. Reason: '{}'.", msg))
            }
            Kind::ResponseStatus(status_kind, data) => {
                f.write_str(&format!("Negative response exception. "))?;
                f.write_str(&format!("Status: '{:?}'. ", status_kind))?;

                if let Some(data) = data {
                    let errors: Vec<&str> = data.errors.iter().map(|item| item.as_str()).collect();
                    return f.write_str(&format!("Errors: {:?}. ", errors));
                } else if let Some(ref data) = self.raw_response_data {
                    return f.write_str(&format!("Data: '{:?}'", data));
                }

                f.write_str(".")
            }
            Kind::DataDeserialization => {
                f.write_str(&format!("Body deserialization exception."))
            }
            Kind::Other(msg) => f.write_str(&format!("Unexpected error. Reason: '{}'.", msg)),
        }
    }
}

/// Body for the responses of stasus 400 or 404.
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponseData {
    errors: Vec<String>,
}

/// Identifies the cause of failure.
#[derive(Debug)]
pub enum Kind {
    ///Inability to establish a connection.
    Connection(String),
    /// Negative response from the server.
    ResponseStatus(ResponseStatusKind, Option<ErrorResponseData>),
    /// The response body for a correctly performed task cannot be deserialized.
    DataDeserialization,
    /// Unknown cause. Check source method.
    Other(String),
}

/// Identifies common causes of errors from server responses.
#[derive(Debug)]
pub enum ResponseStatusKind {
    Unauthorized,
    PayloadTooLarge,
    BadRequest,
    NotFound,
    InternalServerError,
    Conflict,
    Other(Option<u16>),
}

impl ResponseStatusKind {
    fn from_code(code: Option<reqwest::StatusCode>) -> Self {
        if let None = code {
            return Self::Other(None);
        }

        let code = code.unwrap();

        if code.is_server_error() {
            return Self::InternalServerError;
        }

        let code_number = code.as_u16();

        match code_number {
            401 => Self::Unauthorized,
            413 => Self::PayloadTooLarge,
            400 => Self::BadRequest,
            404 => Self::NotFound,
            409 => Self::Conflict,
            _ => Self::Other(Some(code_number)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_response() {
        let error = Error::from_response_data(Some(reqwest::StatusCode::BAD_REQUEST), None, None);
        assert_eq!(error.is_response(), true);
    }

    #[test]
    fn is_not_found() {
        let error = Error::from_response_data(Some(reqwest::StatusCode::NOT_FOUND), None, None);
        assert_eq!(error.is_not_found(), true);
    }

    #[test]
    fn is_bad_request() {
        let error = Error::from_response_data(Some(reqwest::StatusCode::BAD_REQUEST), None, None);
        assert_eq!(error.is_bad_request(), true);
    }

    #[test]
    fn is_body_deserialization() {
        let error = Error {
            kind: Kind::DataDeserialization,
            source: None,
            raw_response_data: None,
        };
        assert_eq!(error.is_body_deserialization(), true);
    }

    #[test]
    fn get_kind() {
        let error = Error::from_response_data(Some(reqwest::StatusCode::BAD_REQUEST), None, None);
        assert!(matches!(
            error.get_kind(),
            Kind::ResponseStatus(ResponseStatusKind::BadRequest, None)
        ));
    }

    #[test]
    fn crate_response_kind_from_code_for_internal_server_error() {
        let code = reqwest::StatusCode::BAD_GATEWAY;
        assert!(matches!(
            ResponseStatusKind::from_code(Some(code)),
            ResponseStatusKind::InternalServerError,
        ))
    }

    #[test]
    fn crate_response_kind_from_code_for_none() {
        assert!(matches!(
            ResponseStatusKind::from_code(None),
            ResponseStatusKind::Other(None),
        ))
    }

    #[test]
    fn crate_response_kind_from_code_for_unexpected_status() {
        let code = reqwest::StatusCode::PROCESSING;
        assert!(matches!(
            ResponseStatusKind::from_code(Some(code)),
            ResponseStatusKind::Other(Some(102)),
        ))
    }

    #[test]
    fn crate_response_kind_from_code_for_not_found_status() {
        let code = reqwest::StatusCode::NOT_FOUND;
        assert!(matches!(
            ResponseStatusKind::from_code(Some(code)),
            ResponseStatusKind::NotFound,
        ))
    }

    #[test]
    fn get_raw_response_data() {
        let error = Error {
            kind: Kind::DataDeserialization,
            source: None,
            raw_response_data: Some("<h1>Some raw response data</h1>".into()),
        };

        assert_eq!(
            error.get_raw_response_data(),
            Some("<h1>Some raw response data</h1>")
        )
    }
}
