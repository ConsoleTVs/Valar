use crate::http::Response;
use crate::http::StatusCode;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use thiserror::Error;

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
    headers: Option<HashMap<String, String>>,
}

impl ErrorResponse {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn status(mut self, status: StatusCode) -> Self {
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

    pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
        self.headers = Some(headers);

        self
    }

    #[cfg(feature = "json")]
    pub fn to_json_response(&self) -> Response {
        let message = self
            .message
            .clone()
            .unwrap_or_else(|| "Internal Server Error".to_string());

        let mut headers =
            HashMap::from([("Content-Type".to_string(), "application/json".to_string())]);

        headers.extend(self.headers.clone().unwrap_or_default());

        let error = JsonError { message };

        Response::builder()
            .status(self.status)
            .headers(headers)
            .json_or(&error, format!(r#"{{ "message": "{}" }}"#, error.message))
            .build()
    }

    pub fn to_response(&self) -> Response {
        let message = self
            .message
            .clone()
            .unwrap_or_else(|| "Internal Server Error".to_string());

        Response::builder()
            .status(self.status)
            .headers(self.headers.clone().unwrap_or_default())
            .body(message)
            .build()
    }
}

impl Default for ErrorResponse {
    fn default() -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: None,
            headers: None,
        }
    }
}
