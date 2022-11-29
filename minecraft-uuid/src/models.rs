use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct UUIDResponse {
    pub name: String,
    pub id: String,
}

#[derive(Deserialize)]
pub(crate) struct ErrorResponse {
    pub error: String,
    #[serde(rename = "errorMessage")]
    pub error_message: String,
}
