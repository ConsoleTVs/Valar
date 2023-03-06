use serde::Deserialize;
use serde::Serialize;
use thiserror::Error;

use crate::http::headers::Headers;
use crate::http::Response;
use crate::http::StatusCode;

#[derive(Serialize, Deserialize)]
pub struct JsonError {
    message: String,
}

#[derive(Error, Debug)]
#[error("{:?}", .message)]
pub struct ErrorResponse {
    /// The status code of the response.
    status: StatusCode,

    /// The message to include in the response.
    message: Option<String>,

    /// Additional Headers to include in the response.
    headers: Option<Headers>,
}

impl ErrorResponse {
    pub const fn new() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: None,
            headers: None,
        }
    }

    pub const fn status(mut self, status: StatusCode) -> Self {
        self.status = status;

        self
    }

    pub fn message<M>(mut self, message: M) -> Self
    where
        M: Into<String>,
    {
        self.message = Some(message.into());

        self
    }

    pub fn headers<H>(mut self, headers: H) -> Self
    where
        H: Into<Headers>,
    {
        self.headers = Some(headers.into());

        self
    }

    pub fn into_json_response(self) -> Response {
        let message = self.message.unwrap_or_else(|| {
            self.status
                .canonical_reason()
                .unwrap_or("Whops, there was an error.")
                .to_string()
                .trim_matches('\t')
        });

        let mut headers = Headers::from([("Content-Type", "application/json")]);

        if let Some(other) = self.headers {
            headers.extend(other);
        }

        let error = JsonError { message };

        Response::builder()
            .status(self.status)
            .headers(headers)
            .json_or(&error, format!(r#"{{ "message": "{}" }}"#, error.message))
            .build()
    }

    pub fn into_response(self) -> Response {
        let message = self.message.unwrap_or_else(|| {
            self.status
                .canonical_reason()
                .unwrap_or("Whops, there was an error.")
                .to_string()
        });

        Response::builder()
            .status(self.status)
            .headers(self.headers.unwrap_or_default())
            .body(message)
            .build()
    }
}
