# minecraft-uuid

A simple API wrapper to convert Minecraft usernames to UUIDs and vice versa.

```toml
[dependencies]
minecraft-uuid = "1"
```

## Example

```rust
use minecraft_uuid::{get_uuid_by_username, get_username_by_uuid};

#[tokio::main]
async fn main() {
    let uuid = get_uuid_by_username("zekrotja")
        .await
        .expect("getting uuid");
    assert_eq!(uuid, "c3371e36f2884eaeb9d5b90e47258444");

    let username = get_username_by_uuid("c3371e36f2884eaeb9d5b90e47258444")
        .await
        .expect("getting username");
    assert_eq!(username, "zekroTJA");
}
```