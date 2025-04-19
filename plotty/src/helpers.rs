use std::fmt;

use serenity::{
    async_trait,
    builder::CreateEmbed,
    http::Http,
    json::Value,
    model::prelude::{
        interaction::application_command::{ApplicationCommandInteraction, CommandDataOption},
        Message,
    },
    utils::Color,
    Result,
};

use crate::models::Region;

#[async_trait]
pub trait FollowUpHelper {
    async fn followup_err<D: ToString + Send + Sync>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        msg: D,
    ) -> Result<Message>;
    async fn followup_embed(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        embed: CreateEmbed,
    ) -> Result<Message>;
    async fn followup<D: ToString + Send + Sync>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        msg: D,
    ) -> Result<Message>;
}

#[async_trait]
impl FollowUpHelper for ApplicationCommandInteraction {
    async fn followup_err<D: ToString + Send + Sync>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        msg: D,
    ) -> Result<Message> {
        self.followup_embed(
            http,
            CreateEmbed::default()
                .color(Color::RED)
                .description(msg)
                .to_owned(),
        )
        .await
    }

    async fn followup_embed(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        embed: CreateEmbed,
    ) -> Result<Message> {
        self.create_followup_message(http, |m| m.ephemeral(true).add_embed(embed))
            .await
    }

    async fn followup<D: ToString + Send + Sync>(
        &self,
        http: impl AsRef<Http> + Send + Sync,
        msg: D,
    ) -> Result<Message> {
        self.followup_embed(
            http,
            CreateEmbed::default()
                .color(Color::FOOYOO)
                .description(msg)
                .to_owned(),
        )
        .await
    }
}

pub trait OptionsHelper {
    fn get_option_by_name(&self, name: &str) -> anyhow::Result<&Value>;
}

impl OptionsHelper for CommandDataOption {
    fn get_option_by_name(&self, name: &str) -> anyhow::Result<&Value> {
        let i = self
            .options
            .iter()
            .find(|o| o.name == name)
            .ok_or_else(|| anyhow::anyhow!("No value for {name}"))?
            .value
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No value for {name}"))?;
        Ok(i)
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "`{}` ({}m²)", self.name, self.perimeter.size())
    }
}
