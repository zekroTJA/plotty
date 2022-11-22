use serenity::{
    builder::CreateEmbed,
    http::Http,
    model::prelude::{interaction::application_command::ApplicationCommandInteraction, Message},
    utils::Color,
    Result,
};

pub async fn followup_err<D: ToString>(
    command: &ApplicationCommandInteraction,
    http: impl AsRef<Http>,
    msg: D,
) -> Result<Message> {
    command
        .create_followup_message(http, |m| {
            m.ephemeral(true).add_embed(
                CreateEmbed::default()
                    .color(Color::RED)
                    .description(msg)
                    .to_owned(),
            )
        })
        .await
}

pub async fn followup<D: ToString>(
    command: &ApplicationCommandInteraction,
    http: impl AsRef<Http>,
    msg: D,
) -> Result<Message> {
    command
        .create_followup_message(http, |m| {
            m.ephemeral(true).add_embed(
                CreateEmbed::default()
                    .color(Color::FOOYOO)
                    .description(msg)
                    .to_owned(),
            )
        })
        .await
}
