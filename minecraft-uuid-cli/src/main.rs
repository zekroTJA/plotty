use std::env;

use minecraft_uuid::{get_username_by_uuid, get_uuid_by_username};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let uuid_or_name = args.get(1);

    if uuid_or_name.is_none() {
        return Err("Please specify a Minecraft username or UUID as first argument.".into());
    }

    let uuid_or_name = uuid_or_name.unwrap();

    let uuid = Uuid::parse_str(uuid_or_name).ok();

    if let Some(uuid) = uuid {
        let res = get_username_by_uuid(&uuid.to_string())
            .await
            .map_err(|e| e.to_string())?;
        println!("{res}");
    } else {
        let res = get_uuid_by_username(uuid_or_name)
            .await
            .map_err(|e| e.to_string())?;
        println!("{res}");
    }

    Ok(())
}
