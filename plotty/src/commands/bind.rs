use crate::db::Database;
use crate::helpers::FollowUpHelper;
use crate::idcache::get_uuid_by_username;
use crate::mc::Rcon;
use anyhow::Result;
use minecraft_client_rs::Message;
use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::ApplicationCommandInteraction;
use serenity::prelude::Context;

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("bind")
        .description("Bind your Discord account to a Minecraft username.")
        .create_option(|o| {
            o.name("username")
                .description("Your ingame Minecraft username.")
                .kind(CommandOptionType::String)
                .required(true)
        })
}

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let mcname = &command
        .data
        .options
        .iter()
        .find(|o| o.name == "username")
        .ok_or_else(|| anyhow::anyhow!("Username option could not be found"))?
        .value
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Username value is empty"))?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Username value is not a string"))?
        .to_lowercase();

    let uuid = get_uuid_by_username(mcname).await?;

    let res = db.get_user_by_mcname(&uuid).await?;

    if let Some(res) = res {
        if res == u64::from(command.user.id) {
            command
                .followup_err(
                    &ctx.http,
                    "This username is already registered by another user.",
                )
                .await?;
            return Ok(());
        }
    }

    let name_before = db.get_user_by_id(command.user.id).await?;

    let msg: Message;
    {
        let mut rc = rc
            .get_conn()
            .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

        if let Some(name_before) = name_before {
            rc.cmd(&format!("whitelist remove {name_before}"))
                .map_err(|e| anyhow::anyhow!(e.to_string()))?;
        }

        msg = rc
            .cmd(&format!("whitelist add {mcname}"))
            .map_err(|e| anyhow::anyhow!(e.to_string()))?;
    }

    if msg.body.trim() == "That player does not exist" {
        command
            .followup_err(&ctx.http, "That Minecraft player name does not not exist.")
            .await?;
        return Ok(());
    }

    db.set_user(command.user.id, &uuid).await?;

    command
        .followup(
            &ctx.http,
            format!(
                concat!(
                    "Successfully bound your account to the name {}. 🥳\n\n",
                    "By the way, you are now also white listed on the server! 👀"
                ),
                mcname
            ),
        )
        .await?;

    Ok(())
}
