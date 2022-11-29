use std::{error::Error, fmt::Display};

use crate::models::ErrorResponse;

#[derive(Debug, Clone)]
pub struct APIError {
    status_code: u16,
    status: String,
    message: String,
}

impl APIError {
    pub fn status_code(&self) -> u16 {
        self.status_code
    }

    pub fn status(&self) -> &str {
        self.status.as_ref()
    }

    pub fn message(&self) -> &str {
        self.message.as_ref()
    }

    pub(crate) fn set_status_code(&mut self, status_code: u16) {
        self.status_code = status_code;
    }
}

impl Error for APIError {}

impl Display for APIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}): {}",
            self.status_code, self.status, self.message
        )
    }
}

impl From<ErrorResponse> for APIError {
    fn from(resp: ErrorResponse) -> Self {
        Self {
            status_code: 0,
            message: resp.error_message,
            status: resp.error,
        }
    }
}
