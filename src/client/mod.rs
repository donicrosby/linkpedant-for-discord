use crate::LinkPedantCommands;
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::model::application::{Command, Interaction};
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use serenity_commands::Commands;
use tracing::{info, instrument, warn};

pub(crate) struct Handler;

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx, _ready))]
    async fn ready(&self, ctx: Context, _ready: Ready) {
        info!("Link Pedant ready!");
        let cmds = LinkPedantCommands::create_commands();
        let _ = Command::set_global_commands(&ctx.http, cmds.clone())
            .await
            .map_err(|err| warn! {%err, "could not create global commands"});
    }

    #[instrument(skip(self, ctx, interaction))]
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::Command(command) = interaction {
            info! {
                command_name = %command.data.name,
                user = %command.user.id,
                guild = %command.guild_id.map(|g|g.get().to_string()).unwrap_or(String::from("None")),
                "handling command",
            };
            let command_data = LinkPedantCommands::from_command_data(&command.data)
                .map_err(|err| warn! {%err, "could not parse interaction command"})
                .unwrap();
            let client_id = ctx.http.application_id().expect("no application id is set");
            command
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content(command_data.run(client_id).await),
                    ),
                )
                .await
                .map_err(|err| warn! {%err, "could not send response"})
                .unwrap();
        }
    }
}
