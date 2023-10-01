mod commands;
mod conf;
mod db;
mod helpers;
mod idcache;
mod mc;
mod models;

use config::{builder::DefaultState, Config, ConfigBuilder, Environment, File, FileFormat};
use db::Database;
use env_logger::Env;
use log::{debug, error, info};
use mc::Rcon;
use serenity::{
    async_trait,
    builder::CreateEmbed,
    model::{
        application::interaction::{
            application_command::ApplicationCommandInteraction,
            autocomplete::AutocompleteInteraction,
        },
        prelude::{
            interaction::{Interaction, InteractionResponseType},
            GuildId, Ready,
        },
    },
    prelude::{Context, EventHandler, GatewayIntents},
    utils::Color,
    Client,
};
use std::{io, sync::Arc};

struct Handler {
    cfg: conf::Config,
    db: Arc<Database>,
    rc: Arc<Rcon>,
}

impl Handler {
    fn new(cfg: conf::Config, db: Arc<Database>, rc: Arc<Rcon>) -> Self {
        Self { cfg, db, rc }
    }
}

impl Handler {
    async fn handle_application_command(
        &self,
        ctx: Context,
        command: ApplicationCommandInteraction,
    ) {
        // Defer interaction emphemerally
        if let Err(err) = command
            .create_interaction_response(&ctx.http, |i| {
                i.kind(InteractionResponseType::DeferredChannelMessageWithSource)
                    .interaction_response_data(|d| d.ephemeral(true))
            })
            .await
        {
            error!("Defering interaction failed: {}", err);
            return;
        }

        debug!("Received command interaction: {:#?}", command);
        let res = match command.data.name.as_str() {
            "region" => commands::region::run(&ctx, &command, &self.db, &self.rc).await,
            "bind" => commands::bind::run(&ctx, &command, &self.db, &self.rc).await,
            _ => Err(anyhow::anyhow!("not implemented")),
        };

        if let Err(err) = res {
            let res = command
                .create_followup_message(&ctx.http, |response| {
                    response.ephemeral(true);
                    response.add_embed(
                        CreateEmbed::default()
                            .color(Color::RED)
                            .description("The command execution failed.")
                            .field("Error", err.to_string(), false)
                            .to_owned(),
                    )
                })
                .await;

            if let Err(err) = res {
                error!("Failed responding command error message: {}", err);
            }
        }
    }

    async fn handle_autocomplete(&self, ctx: Context, autocomplete: AutocompleteInteraction) {
        let _ = match autocomplete.data.name.as_str() {
            "region" => commands::region::autocomplete(&ctx, &autocomplete, &self.db).await,
            "bind" => Ok(()),
            _ => Ok(()),
        };
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        match interaction {
            Interaction::ApplicationCommand(command) => {
                self.handle_application_command(ctx, command).await
            }
            Interaction::Autocomplete(autocomplete) => {
                self.handle_autocomplete(ctx, autocomplete).await
            }
            _ => {}
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        info!("Client ready (logged in as {})", ready.user.tag());

        let guild_id = GuildId(self.cfg.discord.guildid);

        GuildId::set_application_commands(&guild_id, &ctx.http, |commands| {
            commands
                .create_application_command(|command| commands::region::register(command))
                .create_application_command(|command| commands::bind::register(command))
        })
        .await
        .expect("Command registration failed");
    }
}

#[tokio::main]
async fn main() {
    env_logger::Builder::from_env(
        Env::default().default_filter_or("info,serenity=warn,tracing=warn,sqlx::query=warn"),
    )
    .try_init()
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    .expect("Failed building logger");

    let mut b = Config::builder();

    b = add_source_files_for_dir(b, "config");
    b = add_source_files_for_dir(b, "dev.config");
    b = b.add_source(Environment::with_prefix("PLOTTY").separator("_"));

    info!("Reading config ...");
    let cfg: conf::Config = b
        .build()
        .expect("Failed to build config")
        .try_deserialize()
        .expect("Failed deserializing config");

    info!("Initializing database ...");
    let db = Arc::new(
        Database::new(&cfg.database.dsn)
            .await
            .expect("Database initialization failed"),
    );

    info!("Preparing database ...");
    db.init().await.expect("Database preparation failed");

    info!("Initializing RCON connection ...");
    let rc = Arc::new(Rcon::new(&cfg.rcon).expect("RCON client initialization failed"));

    let mut client = Client::builder(cfg.discord.token.clone(), GatewayIntents::empty())
        .event_handler(Handler::new(cfg.clone(), db.clone(), rc.clone()))
        .await
        .expect("Failed creating Discord client");

    info!("Initializing Discord connection ...");
    client
        .start()
        .await
        .expect("Failed starting Discord client");
}

fn add_source_files_for_dir(
    builder: ConfigBuilder<DefaultState>,
    name: &str,
) -> ConfigBuilder<DefaultState> {
    builder
        .add_source(File::new(&format!("{name}.yaml"), FileFormat::Yaml).required(false))
        .add_source(File::new(&format!("{name}.yml"), FileFormat::Yaml).required(false))
        .add_source(File::new(&format!("{name}.toml"), FileFormat::Toml).required(false))
        .add_source(File::new(&format!("{name}.json"), FileFormat::Json5).required(false))
}
