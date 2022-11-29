use anyhow::Result;
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::sync::OnceCell;

static CACHE: OnceCell<Arc<RwLock<HashMap<String, String>>>> = OnceCell::const_new();

fn get_cache() -> &'static Arc<RwLock<HashMap<String, String>>> {
    if !CACHE.initialized() {
        CACHE
            .set(Arc::new(RwLock::new(HashMap::new())))
            .expect("setting cache map");
    }

    CACHE.get().expect("getting cache map")
}

pub async fn get_username_by_uuid(uuid: &str) -> Result<String> {
    {
        let cache = get_cache().read().expect("cache map is poisoned");
        if let Some(name) = cache.get(uuid) {
            return Ok(name.to_owned());
        }
    }

    let name = minecraft_uuid::get_username_by_uuid(uuid).await?;

    let mut cache = get_cache().write().expect("cache map is poisoned");
    cache.insert(uuid.to_owned(), name.clone());

    Ok(name)
}

pub async fn get_uuid_by_username(username: &str) -> Result<String> {
    {
        let cache = get_cache().read().expect("cache map is poisoned");
        if let Some((_, uuid)) = cache.iter().find(|(_, v)| v == &username) {
            return Ok(uuid.to_owned());
        }
    }

    let uuid = minecraft_uuid::get_uuid_by_username(username).await?;

    let mut cache = get_cache().write().expect("cache map is poisoned");
    cache.insert(uuid.clone(), username.to_owned());

    Ok(uuid)
}
