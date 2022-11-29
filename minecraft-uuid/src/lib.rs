mod endpoints;
pub mod error;
mod models;

use anyhow::Result;
use error::APIError;
use models::{ErrorResponse, UUIDResponse};
use serde::de::DeserializeOwned;

/// Get a UUID from the given Minecraft username
/// from the Mojang API.
///
/// # Example
/// ```
/// # use minecraft_uuid::*;
/// # async fn run() -> anyhow::Result<()> {
/// let uuid = get_uuid_by_username("zekrotja").await?;
/// assert_eq!(uuid, "c3371e36f2884eaeb9d5b90e47258444");
/// # Ok(())
/// # }
/// ```
pub async fn get_uuid_by_username(username: &str) -> Result<String> {
    let res: UUIDResponse = get(&endpoints::usernames(username)).await?;
    Ok(res.id)
}

/// Get a Minecraft username from the given account UUID
/// from the Mojang API.
///
/// # Example
/// ```
/// # use minecraft_uuid::*;
/// # async fn run() -> anyhow::Result<()> {
/// let username = get_username_by_uuid("c3371e36f2884eaeb9d5b90e47258444").await?;
/// assert_eq!(username, "zekroTJA");
/// # Ok(())
/// # }
/// ```
pub async fn get_username_by_uuid(uuid: &str) -> Result<String> {
    let res: UUIDResponse = get(&endpoints::uids(uuid)).await?;
    Ok(res.name)
}

async fn get<T: DeserializeOwned>(url: &str) -> Result<T> {
    let resp = reqwest::get(url).await?;

    let status = u16::from(resp.status());
    if status == 204 {
        let err = APIError::new(404, "NotFound", "This user does not exist.");
        return Err(err.into());
    } else if status > 399 {
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

        let res = get_uuid_by_username("shit").await;
        let err = res.unwrap_err();
        let err = err.downcast_ref::<APIError>().unwrap();
        assert_eq!(err.status_code(), 404);

        let res = get_uuid_by_username("this should return a bad request i suppose").await;
        let err = res.unwrap_err();
        let err = err.downcast_ref::<APIError>().unwrap();
        assert_eq!(err.status_code(), 400);
    }

    #[tokio::test]
    async fn test_get_username_by_uuid() {
        let res = get_username_by_uuid("c3371e36f2884eaeb9d5b90e47258444").await;
        assert_eq!(res.unwrap(), "zekroTJA");

        let res = get_username_by_uuid("invaliduuid").await;
        let err = res.unwrap_err();
        let err = err.downcast_ref::<APIError>().unwrap();
        assert_eq!(err.status_code(), 400);

        let res = get_username_by_uuid("79fc2caa329a4769bf47aaf351684d71").await;
        let err = res.unwrap_err();
        let err = err.downcast_ref::<APIError>().unwrap();
        assert_eq!(err.status_code(), 404);
    }
}
