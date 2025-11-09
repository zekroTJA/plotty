use crate::db::Database;
use crate::helpers::{FollowUpHelper, OptionsHelper};
use crate::idcache::get_username_by_uuid;
use crate::mc::{Conn, Rcon};
use crate::models::{Perimeter, Point, Region};
use anyhow::{bail, Result};
use minecraft_client_rs::Message;
use serenity::builder::{CreateApplicationCommand, CreateEmbed};
use serenity::futures::future::join_all;
use serenity::json::json;
use serenity::model::prelude::autocomplete::AutocompleteInteraction;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::component::ButtonStyle;
use serenity::model::prelude::interaction::application_command::{
    ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::user::User;
use serenity::prelude::Context;
use serenity::utils::Color;
use std::error::Error;
use std::time::Duration;

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
                .create_sub_option(|so| {
                    so.name("world")
                        .description("The world to create the plot in")
                        .kind(CommandOptionType::String)
                        .add_string_choice("Overworld", "world")
                        .add_string_choice("Nether", "nether")
                        .add_string_choice("The End", "the_end")
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
                        .set_autocomplete(true)
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
                .create_sub_option(|so| {
                    so.name("world")
                        .description("The world to create the plot in")
                        .kind(CommandOptionType::String)
                        .add_string_choice("Overworld", "world")
                        .add_string_choice("Nether", "nether")
                        .add_string_choice("The End", "the_end")
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
                                .set_autocomplete(true)
                        })
                        .create_sub_option(|sso| {
                            sso.name("username")
                                .description("The Minecraft name of the member to be added.")
                                .kind(CommandOptionType::String)
                                .required(true)
                                .set_autocomplete(true)
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
                                .set_autocomplete(true)
                        })
                        .create_sub_option(|sso| {
                            sso.name("username")
                                .description("The Minecraft name of the member to be removed.")
                                .kind(CommandOptionType::String)
                                .required(true)
                                .set_autocomplete(true)
                        })
                })
        })
        // ----------------------------------
        // delete sub command
        .create_option(|o| {
            o.name("delete")
                .description("Delete your personal region")
                .kind(CommandOptionType::SubCommand)
                .create_sub_option(|so| {
                    so.name("plotname")
                        .description("The name of your plot.")
                        .kind(CommandOptionType::String)
                        .required(true)
                        .set_autocomplete(true)
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

    let username = get_username_by_uuid(&res.unwrap()).await?.to_lowercase();

    let options = &command.data.options;
    let subcmd = options
        .first()
        .ok_or_else(|| anyhow::anyhow!("Response does not contain any sub command option."))?;

    match subcmd.name.as_str() {
        "list" => list(ctx, command, db).await,
        "create" => create(ctx, command, subcmd, &username, db, rc).await,
        "redefine" => redefine(ctx, command, subcmd, db, rc).await,
        "members" => members(ctx, command, subcmd, db, rc).await,
        "delete" => delete(ctx, command, subcmd, db, rc).await,
        _ => Err(anyhow::anyhow!("Unregistered sub command")),
    }
}

fn find_option_deep(i: &AutocompleteInteraction, name: &str) -> Option<CommandDataOption> {
    i.data
        .options
        .iter()
        .flat_map(|s| &s.options)
        .flat_map(
            |s| {
                if s.options.is_empty() {
                    vec![s.clone()]
                } else {
                    s.options.clone()
                }
            },
        )
        .find(|o| o.name == name && o.focused)
}

async fn get_user(ctx: &Context, id: u64) -> Result<User> {
    match ctx.cache.user(id) {
        Some(u) => Ok(u),
        None => Ok(ctx.http.get_user(id).await?),
    }
}

pub async fn autocomplete(ctx: &Context, i: &AutocompleteInteraction, db: &Database) -> Result<()> {
    if let Some(plotname) = find_option_deep(i, "plotname") {
        let plots = db
            .get_user_plots(i.user.id)
            .await?
            .iter()
            .filter(|p| {
                plotname
                    .value
                    .as_ref()
                    .and_then(|v| v.as_str())
                    .is_some_and(|v| p.name.starts_with(v))
            })
            .map(|p| {
                json!({
                    "name": p.name,
                    "value": p.name
                })
            })
            .collect();

        i.create_autocomplete_response(&ctx.http, |r| r.set_choices(plots))
            .await?;
    }

    if let Some(username) = find_option_deep(i, "username") {
        let res = db.list_users().await?;

        let users = join_all(res.iter().map(|u| async {
            let uname = get_user(ctx, u.discord_id)
                .await
                .map(|u| u.name)
                .unwrap_or_else(|_| u.discord_id.to_string());
            (u.clone(), uname)
        }))
        .await;

        let usernames = users
            .iter()
            .filter(|(u, uname)| {
                u.discord_id != i.user.id.0
                    && username
                        .value
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .is_some_and(|v| uname.starts_with(v))
            })
            .map(|(u, uname)| async {
                minecraft_uuid::get_username_by_uuid(&u.minecraft_uid)
                    .await
                    .ok()
                    .map(|mc_uname| {
                        json!({
                            "name": format!("{} ({})", uname.clone(), &mc_uname),
                            "value": mc_uname,
                        })
                    })
            });

        let usernames = join_all(usernames)
            .await
            .iter()
            .filter_map(|r| r.clone())
            .collect();

        i.create_autocomplete_response(
            &ctx.http,
            |r: &mut serenity::builder::CreateAutocompleteResponse| r.set_choices(usernames),
        )
        .await?;
    }

    Ok(())
}

// ---- SUB COMMAND HANDLERS ----

async fn list(ctx: &Context, command: &ApplicationCommandInteraction, db: &Database) -> Result<()> {
    let plots = db
        .get_user_plots(command.user.id)
        .await?
        .iter()
        .map(|p| format!("  ▫️ {}", p))
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
    let plot_id = db
        .get_plot_user_id(command.user.id)
        .await?
        .unwrap_or_default();

    let plot_name = format!("{}_plot_{}", username.replace('_', ""), plot_id + 1);

    let world = subcmd
        .get_option_by_name("world")?
        .as_str()
        .unwrap_or("world");

    let perimeter = Perimeter(
        Point(
            get_pos_option(subcmd, "pos1-x")?,
            get_pos_option(subcmd, "pos1-z")?,
        ),
        Point(
            get_pos_option(subcmd, "pos2-x")?,
            get_pos_option(subcmd, "pos2-z")?,
        ),
    );

    let collisions = find_collisions(db, command.user.id.into(), &perimeter).await?;
    if !collisions.is_empty() {
        anyhow::bail!(
            "The perimeter of your defined plot would collide with {} other plot{}!",
            collisions.len(),
            if collisions.len() > 1 { "s" } else { "" }
        );
    }

    let region = Region {
        owner: command.user.id.into(),
        name: plot_name.clone(),
        perimeter,
    };

    db.inc_plot_user_id(command.user.id).await?;
    create_plot(rc, &region, username, world)?;
    db.add_plot(&region).await?;

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

    let world = subcmd
        .get_option_by_name("world")?
        .as_str()
        .unwrap_or("world");

    let region = db.get_plot_by_name(plot_name).await?;
    if region.is_none() || region.unwrap().owner != u64::from(command.user.id) {
        command
            .followup_err(&ctx.http, "You can not update this plot.")
            .await?;
        return Ok(());
    }

    let perimeter = Perimeter(
        Point(
            get_pos_option(subcmd, "pos1-x")?,
            get_pos_option(subcmd, "pos1-z")?,
        ),
        Point(
            get_pos_option(subcmd, "pos2-x")?,
            get_pos_option(subcmd, "pos2-z")?,
        ),
    );

    let collisions = find_collisions(db, command.user.id.into(), &perimeter).await?;
    if !collisions.is_empty() {
        anyhow::bail!(
            "The perimeter of your defined plot would collide with {} other plot{}!",
            collisions.len(),
            if collisions.len() > 1 { "s" } else { "" }
        );
    }

    let region = Region {
        owner: command.user.id.into(),
        name: plot_name.clone(),
        perimeter,
    };

    update_plot(rc, &region, world)?;
    db.update_plot(&region).await?;

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
        .first()
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
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?
        .to_lowercase();

    let membername = subcmd
        .get_option_by_name("username")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?;

    let region = db.get_plot_by_name(&plotname).await?;

    if region.is_none() || region.unwrap().owner != u64::from(command.user.id) {
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
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?
        .to_lowercase();

    let membername = subcmd
        .get_option_by_name("username")?
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Value is not a string"))?;

    let region = db.get_plot_by_name(&plotname).await?;

    if region.is_none() || region.unwrap().owner != u64::from(command.user.id) {
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

async fn delete(
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

    let region = db.get_plot_by_name(plot_name).await?;
    if region.is_none() || region.unwrap().owner != u64::from(command.user.id) {
        command
            .followup_err(&ctx.http, "You can not delete this plot.")
            .await?;
        return Ok(());
    }

    let ok_id = xid::new().to_string();
    let cancel_id = xid::new().to_string();
    let m = command
        .create_followup_message(&ctx.http, |msg| {
            msg.add_embed(
                CreateEmbed::default()
                    .description(format!(
                        "Do you really want to delete your plot {plot_name}?"
                    ))
                    .color(Color::ORANGE)
                    .to_owned(),
            )
            .components(|c| {
                c.create_action_row(|row| {
                    row.create_button(|btn| {
                        btn.custom_id(&ok_id)
                            .style(ButtonStyle::Danger)
                            .label("Delete Plot")
                    })
                    .create_button(|btn| {
                        btn.custom_id(&cancel_id)
                            .style(ButtonStyle::Secondary)
                            .label("Cancel")
                    })
                })
            })
        })
        .await?;

    let interaction = m
        .await_component_interaction(ctx)
        .timeout(Duration::from_secs(60))
        .await
        .ok_or_else(|| anyhow::anyhow!("Timed out."))?;

    if interaction.data.custom_id == cancel_id {
        interaction
            .create_interaction_response(&ctx.http, |r| {
                r.kind(InteractionResponseType::UpdateMessage)
                    .interaction_response_data(|d| {
                        d.add_embed(
                            CreateEmbed::default()
                                .description("Action canceled.")
                                .to_owned(),
                        )
                        .components(|c| c)
                    })
            })
            .await?;
        return Ok(());
    }

    {
        let mut conn = rc
            .get_conn()
            .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

        // TODO: Make world configurable
        check_err(conn.cmd(&format!("rg delete -w world {plot_name}")))?;
    }

    db.delete_plot(plot_name).await?;

    interaction
        .create_interaction_response(&ctx.http, |r| {
            r.kind(InteractionResponseType::UpdateMessage)
                .interaction_response_data(|d| {
                    d.add_embed(
                        CreateEmbed::default()
                            .color(Color::FOOYOO)
                            .description("The plot has been deleted.")
                            .to_owned(),
                    )
                    .components(|c| c)
                })
        })
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

fn create_plot(rc: &Rcon, region: &Region, user_name: &str, world: &str) -> Result<()> {
    let mut conn = rc
        .get_conn()
        .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

    select_perimeter(&mut conn, &region.perimeter, world)?;
    check_err(conn.cmd(&format!("region create {} {}", region.name, user_name)))?;

    Ok(())
}

fn update_plot(rc: &Rcon, region: &Region, world: &str) -> Result<()> {
    let mut conn = rc
        .get_conn()
        .map_err(|e| anyhow::anyhow!("RCON connection failed: {}", e.to_string()))?;

    select_perimeter(&mut conn, &region.perimeter, world)?;
    check_err(conn.cmd(&format!("rg update {}", region.name)))?;

    Ok(())
}

fn select_perimeter(conn: &mut Conn, perimeter: &Perimeter, world: &str) -> Result<()> {
    // TODO: Make configurable or whatever.
    check_err(conn.cmd(&format!("//world {world}")))?;
    check_err(conn.cmd(&format!("//pos1 {},0,{}", perimeter.0 .0, perimeter.0 .1)))?;
    check_err(conn.cmd(&format!("//pos2 {},0,{}", perimeter.1 .0, perimeter.1 .1)))?;
    check_err(conn.cmd("//expand vert"))?;
    Ok(())
}

fn check_err(res: Result<Message, Box<dyn Error>>) -> Result<Message> {
    let msg = res.map_err(|e| anyhow::anyhow!(e.to_string()))?;
    if msg.body.starts_with(ERR_PREFIX) {
        bail!(msg.body);
    }
    Ok(msg)
}

async fn find_collisions(
    db: &Database,
    user_id: u64,
    perimeter: &Perimeter,
) -> Result<Vec<Region>> {
    let plots = db.get_plots().await?;

    let res = plots
        .iter()
        .filter(|p| p.owner != user_id && p.perimeter.intersects(perimeter))
        .cloned()
        .collect();

    Ok(res)
}
