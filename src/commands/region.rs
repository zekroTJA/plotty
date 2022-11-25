use crate::{
    db::Database,
    helpers::{FollowUpHelper, OptionsHelper},
    mc::{Conn, Rcon},
};
use anyhow::{bail, Result};
use minecraft_client_rs::Message;
use serenity::{
    builder::{CreateApplicationCommand, CreateEmbed},
    model::prelude::{
        command::CommandOptionType,
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOption},
    },
    prelude::Context,
    utils::Color,
};
use std::error::Error;

const ERR_PREFIX: &str = "§c";

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
    command
        .name("region")
        .description("Create, update or remove regions.")
        // ----------------------------------
        // list sub command
        .create_option(|o| {
            o.name("list")
                .description("List your plots.")
                .kind(CommandOptionType::SubCommand)
        })
        // ----------------------------------
        // create sub command
        .create_option(|o| {
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
        // ----------------------------------
        // redefine sub command
        .create_option(|o| {
            o.name("redefine")
                .description("Update the perimeter of your personal region")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("plotname")
                        .description("The name of your plot.")
                        .kind(CommandOptionType::String)
                        .required(true)
                })
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
        // ----------------------------------
        // member sub command group
        .create_option(|o| {
            o.name("members")
                .description("Commands to manage plot members.")
                .kind(CommandOptionType::SubCommandGroup)
                .create_sub_option(|so| {
                    so.name("add")
                        .description("Add a member to your plot.")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sso| {
                            sso.name("plotname")
                                .description("The name of the plot.")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                        .create_sub_option(|sso| {
                            sso.name("username")
                                .description("The Minecraft name of the member to be added.")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                })
                .create_sub_option(|so| {
                    so.name("remove")
                        .description("Remove a member from your plot.")
                        .kind(CommandOptionType::SubCommand)
                        .create_sub_option(|sso| {
                            sso.name("plotname")
                                .description("The name of the plot.")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
                        .create_sub_option(|sso| {
                            sso.name("username")
                                .description("The Minecraft name of the member to be removed.")
                                .kind(CommandOptionType::String)
                                .required(true)
                        })
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
        command.followup_err(&ctx.http,
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
        "list" => list(ctx, command, db).await,
        "create" => create(ctx, command, subcmd, &username, db, rc).await,
        "redefine" => redefine(ctx, command, subcmd, db, rc).await,
        "members" => members(ctx, command, subcmd, db, rc).await,
        _ => Err(anyhow::anyhow!("Unregistered sub command")),
    }
}

// ---- SUB COMMAND HANDLERS ----

async fn list(ctx: &Context, command: &ApplicationCommandInteraction, db: &Database) -> Result<()> {
    let plots = db
        .get_user_plots(command.user.id)
        .await?
        .iter()
        .map(|p| format!("  ▫️ {p}"))
        .collect::<Vec<_>>()
        .join("\n");

    command
        .followup_embed(
            &ctx.http,
            CreateEmbed::default()
                .color(Color::BLURPLE)
                .description(format!("These are all your plots:\n\n{plots}"))
                .to_owned(),
        )
        .await?;

    Ok(())
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

    let plot_name = format!(
        "{}_plot_{}",
        username.replace('_', ""),
        plot_names.len() + 1
    );

    let pos1_x = get_pos_option(subcmd, "pos1-x")?;
    let pos1_z = get_pos_option(subcmd, "pos1-z")?;
    let pos2_x = get_pos_option(subcmd, "pos2-x")?;
    let pos2_z = get_pos_option(subcmd, "pos2-z")?;

    create_plot(rc, username, &plot_name, pos1_x, pos1_z, pos2_x, pos2_z)?;
    db.add_plot(command.user.id, &plot_name).await?;

    command
        .followup(
            &ctx.http,
            format!("Your plot `{plot_name}` has been created! 🎉"),
        )
        .await?;

    Ok(())
}

async fn redefine(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    subcmd: &CommandDataOption,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let plot_name = &subcmd
        .get_option_by_name("plotname")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Plot name value is not a string"))?
        .to_lowercase();

    let owner_id = db.get_plot_by_name(plot_name).await?;
    if owner_id.is_none() || owner_id.unwrap() != u64::from(command.user.id) {
        command
            .followup_err(&ctx.http, "You can not update this plot.")
            .await?;
        return Ok(());
    }

    let pos1_x = get_pos_option(subcmd, "pos1-x")?;
    let pos1_z = get_pos_option(subcmd, "pos1-z")?;
    let pos2_x = get_pos_option(subcmd, "pos2-x")?;
    let pos2_z = get_pos_option(subcmd, "pos2-z")?;

    update_plot(rc, plot_name, pos1_x, pos1_z, pos2_x, pos2_z)?;

    command
        .followup(
            &ctx.http,
            format!("The perimeter of your plot `{plot_name}` has been updated! 🎉"),
        )
        .await?;

    Ok(())
}

async fn members(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    subcmd: &CommandDataOption,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let subcmd = subcmd
        .options
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("Response does not contain any sub command option."))?;

    match subcmd.name.as_str() {
        "add" => members_add(ctx, command, subcmd, db, rc).await,
        "remove" => members_remove(ctx, command, subcmd, db, rc).await,
        _ => Err(anyhow::anyhow!("Unregistered sub-sub command")),
    }
}

async fn members_add(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    subcmd: &CommandDataOption,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let plotname = subcmd
        .get_option_by_name("plotname")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?;

    let membername = subcmd
        .get_option_by_name("username")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?;

    let owner_id = db.get_plot_by_name(plotname).await?;

    if owner_id.is_none() || owner_id.unwrap() != u64::from(command.user.id) {
        command
            .followup_err(&ctx.http, "You can not alter the members of this plot.")
            .await?;
        return Ok(());
    }

    {
        let mut conn = rc
            .get_conn()
            .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

        // TODO: Make world value configurable
        check_err(conn.cmd(&format!("rg addmember -w world {plotname} {membername}")))?;
    }

    command
        .followup(
            &ctx.http,
            format!("Member {membername} has been added to plot {plotname}! 🎉"),
        )
        .await?;

    Ok(())
}

async fn members_remove(
    ctx: &Context,
    command: &ApplicationCommandInteraction,
    subcmd: &CommandDataOption,
    db: &Database,
    rc: &Rcon,
) -> Result<()> {
    let plotname = subcmd
        .get_option_by_name("plotname")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?;

    let membername = subcmd
        .get_option_by_name("username")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?;

    let owner_id = db.get_plot_by_name(plotname).await?;

    if owner_id.is_none() || owner_id.unwrap() != u64::from(command.user.id) {
        command
            .followup_err(&ctx.http, "You can not alter the members of this plot.")
            .await?;
        return Ok(());
    }

    {
        let mut conn = rc
            .get_conn()
            .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

        // TODO: Make world value configurable
        check_err(conn.cmd(&format!(
            "rg removemember -w world  {plotname} {membername}"
        )))?;
    }

    command
        .followup(
            &ctx.http,
            format!("Member {membername} has been removed from plot {plotname}!"),
        )
        .await?;

    Ok(())
}

// ---- HELPERS ----

fn get_pos_option(subcmd: &CommandDataOption, name: &str) -> Result<i64> {
    let i = subcmd
        .get_option_by_name(name)?
        .as_i64()
        .ok_or_else(|| anyhow::anyhow!("Value is not of type i64"))?;
    Ok(i)
}

fn create_plot(
    rc: &Rcon,
    user_name: &str,
    plot_name: &str,
    pos1_x: i64,
    pos1_z: i64,
    pos2_x: i64,
    pos2_z: i64,
) -> Result<()> {
    let mut conn = rc
        .get_conn()
        .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

    select_perimeter(&mut conn, pos1_x, pos1_z, pos2_x, pos2_z)?;
    check_err(conn.cmd(&format!("rg create {plot_name} {user_name}")))?;

    Ok(())
}

fn update_plot(
    rc: &Rcon,
    plot_name: &str,
    pos1_x: i64,
    pos1_z: i64,
    pos2_x: i64,
    pos2_z: i64,
) -> Result<()> {
    let mut conn = rc
        .get_conn()
        .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

    select_perimeter(&mut conn, pos1_x, pos1_z, pos2_x, pos2_z)?;
    check_err(conn.cmd(&format!("rg update {plot_name}")))?;

    Ok(())
}

fn select_perimeter(
    conn: &mut Conn,
    pos1_x: i64,
    pos1_z: i64,
    pos2_x: i64,
    pos2_z: i64,
) -> Result<()> {
    // TODO: Make configurable or whatever.
    check_err(conn.cmd("/world world"))?;
    check_err(conn.cmd(&format!("/pos1 {pos1_x},0,{pos1_z}")))?;
    check_err(conn.cmd(&format!("/pos2 {pos2_x},0,{pos2_z}")))?;
    check_err(conn.cmd("/expand vert"))?;
    Ok(())
}

fn check_err(res: Result<Message, Box<dyn Error>>) -> Result<Message> {
    let msg = res.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if msg.body.starts_with(ERR_PREFIX) {
        bail!(msg.body);
    }
    Ok(msg)
}
