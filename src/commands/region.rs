use std::{ error::Error};
use minecraft_client_rs::Message;
use serenity::{
    builder::CreateApplicationCommand,
    model::prelude::{
        command::CommandOptionType,
        interaction::{
            application_command::{ApplicationCommandInteraction, CommandDataOption},
        },
    },
    prelude::Context,
};
use anyhow::{Result, bail};
use crate::{
    db::Database,
    helpers::{followup_err, followup},
    mc::Rcon,
};

const ERR_PREFIX: &str = "§c";

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("region")
        .description("Create, update or remove regions.")
        .create_option(|o| {
            // ----------------------------------
            // create sub command
            o.name("create")
                .description("Create a new personal region")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("pos1-x")
                        .description("The X coordinate of the first corner position.")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
                .create_sub_option(|so| {
                    so.name("pos1-z")
                        .description("The Z coordinate of the first corner position.")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
                .create_sub_option(|so| {
                    so.name("pos2-x")
                        .description("The X coordinate of the second corner position.")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
                .create_sub_option(|so| {
                    so.name("pos2-z")
                        .description("The Y coordinate of the second corner position.")
                        .kind(CommandOptionType::Integer)
                        .required(true)
                })
        })
}

pub async fn run(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let res = db.get_user_by_id(command.user.id).await?;
    if res.is_none() {
        followup_err(command, &ctx.http, 
            "You have not registered a Minecraft username. Please use the `/bind` command to bind your Discord account to your Minecrfat username.")
            .await?;
        return Ok(());
    }

    let username = res.unwrap();

    let options = &command.data.options;
    let subcmd = options
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("Response does not contain any sub command option."))?;

    match subcmd.name.as_str() {
        "create" => create(ctx, command, subcmd, &username, db, rc).await,
        _ => Err(anyhow::anyhow!("Unregistered sub command")),
    }
}

async fn create(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    subcmd: &CommandDataOption,
    username: &str,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let plot_names = db.get_user_plots(command.user.id).await?;

    let plot_name = format!("{}_plot_{}", username.replace('_', ""), plot_names.len() + 1);

    dbg!(subcmd);

    let pos1_x = get_pos_option(subcmd, "pos1-x")?;
    let pos1_z = get_pos_option(subcmd, "pos1-z")?;
    let pos2_x = get_pos_option(subcmd, "pos2-x")?;
    let pos2_z = get_pos_option(subcmd, "pos2-z")?;

    create_plot(rc, username, &plot_name, pos1_x, pos1_z, pos2_x, pos2_z)?;
    db.add_plot(command.user.id, &plot_name).await?;

    followup(command, &ctx.http, format!("Your plot {plot_name} has been created! 🎉")).await?;

    Ok(())
}

fn get_pos_option(subcmd: &CommandDataOption, name: &str) -> Result<i64> {
    let i = subcmd.options
        .iter()
        .find(|o| o.name == name)
        .ok_or_else(|| anyhow::anyhow!("No value for {}", name))?
        .value
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No value for {}", name))?
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Value is not an i64"))?;
    Ok(i)
}

fn create_plot(rc: &Rcon, user_name: &str, plot_name: &str, pos1_x: i64, pos1_z: i64, pos2_x: i64, pos2_z: i64) -> Result<()> {
    let mut rc = rc
            .get_conn()
            .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;
    
    // TODO: Make configurable or whatever.
    check_err(rc.cmd("/world world"))?;
    check_err(rc.cmd(&format!("/pos1 {pos1_x},0,{pos1_z}")))?;
    check_err(rc.cmd(&format!("/pos2 {pos2_x},0,{pos2_z}")))?;
    check_err(rc.cmd("/expand vert"))?;
    check_err(rc.cmd(&format!("rg create {plot_name} {user_name}")))?;

    Ok(())
}

fn check_err(res: Result<Message, Box<dyn Error>>) -> Result<Message> {
    let msg = res.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if msg.body.starts_with(ERR_PREFIX) {
        bail!(msg.body);
    }
    Ok(msg)
}