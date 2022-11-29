mod endpoints;
pub mod error;
mod models;

use anyhow::Result;
use error::APIError;
use models::{ErrorResponse, UUIDResponse};
use serde::de::DeserializeOwned;

pub async fn get_uuid_by_username(username: &str) -> Result<String> {
    let res: UUIDResponse = get(&endpoints::usernames(username)).await?;
    Ok(res.id)
}

pub async fn get_username_by_uuid(uuid: &str) -> Result<String> {
    let res: UUIDResponse = get(&endpoints::uids(uuid)).await?;
    Ok(res.name)
}

async fn get<T: DeserializeOwned>(url: &str) -> Result<T> {
    let resp = reqwest::get(url).await?;

    let status = u16::from(resp.status());
    if status > 399 {
        let body: ErrorResponse = resp.json().await?;
        let mut err: APIError = body.into();
        err.set_status_code(status);
        return Err(err.into());
    }

    let body = resp.json().await?;
    Ok(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_uuid_by_username() {
        let res = get_uuid_by_username("zekrotja").await;
        assert_eq!(res.unwrap(), "c3371e36f2884eaeb9d5b90e47258444");
    }

    #[tokio::test]
    async fn test_get_username_by_uuid() {
        let res = get_username_by_uuid("c3371e36f2884eaeb9d5b90e47258444").await;
        assert_eq!(res.unwrap(), "zekroTJA");
    }
}
