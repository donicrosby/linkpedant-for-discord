use crate::{
    get_invite_command, BotState, DeleteReplyReaction, DeleteReplyReactionConfig,
    LinkPedantCommands, MessageHandler,
};
use serenity::all::{EditMessage, ErrorResponse, Permissions, Reaction, Ready, StatusCode};
use serenity::async_trait;
use serenity::builder::{CreateInteractionResponse, CreateInteractionResponseMessage};
use serenity::futures::{future, TryFutureExt};
use serenity::model::application::{Command, Interaction};
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::fmt::Display;
use thiserror::Error;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Copy, Clone)]
enum NeededPermissions {
    SendMessage,
    EditMessage,
}

impl Display for NeededPermissions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SendMessage => write!(f, "send message"),
            Self::EditMessage => write!(f, "edit message"),
        }
    }
}

#[derive(Debug, Error)]
enum BotClientErrors {
    #[error("no processors in typemap")]
    NoProcessors,
    #[error("message not modified")]
    NotModified,
    #[error("insufficient permissions: `{0}`")]
    InsufficientPermissions(NeededPermissions),
    #[error("no delete emoji in typemap")]
    NoDeleteReply,
    #[error("not my message")]
    NotMyMessage,
    #[error("invalid delete emoji")]
    InvalidEmoji,
    #[error("no replied message")]
    NoReply,
    #[error("not original author")]
    NotOriginalAuthor,
    #[error("serenity error: {0}")]
    Serenity(#[from] SerenityError),
}

pub(crate) struct Handler;

impl Handler {
    async fn message_handler(&self, ctx: Context, message: Message) -> Result<(), BotClientErrors> {
        self.process_message(&ctx, message)
            .and_then(|(ctx, original_message, reply)| async move {
                debug!("was able to process message, replying...");
                original_message
                    .reply(ctx, reply)
                    .map_err(Self::parse_errors)
                    .await
                    .map(|reply| (ctx, original_message, reply))
            })
            .and_then(|(ctx, message, reply)| async move {
                let mut message = message;
                let suppress_embeds = EditMessage::new().suppress_embeds(true);
                if let Err(err) = message
                    .edit(ctx, suppress_embeds)
                    .map_err(Self::parse_errors)
                    .await
                {
                    warn!("unable to edit original message, cleaning up...");
                    match reply.delete(ctx).map_err(Self::parse_errors).await {
                        Ok(_) => Err(err),
                        Err(err) => {
                            warn!("could not clean up reply message...");
                            Err(err)
                        }
                    }
                } else {
                    Ok(())
                }
            })
            .and_then(|_| async move {
                debug!("finished processing");
                Ok(())
            })
            .await
    }

    async fn get_reaction_message<'a>(
        &self,
        ctx: &'a Context,
        reaction: Reaction,
    ) -> Result<(&'a Context, Message, DeleteReplyReaction), BotClientErrors> {
        let emoji = self.get_reaction_emoji(ctx).await?;

        reaction
            .message(&ctx)
            .map_err(|e| e.into())
            .await
            .map(|msg| (ctx, msg, emoji))
    }

    async fn get_reaction_emoji(
        &self,
        ctx: &Context,
    ) -> Result<DeleteReplyReaction, BotClientErrors> {
        let data_read = ctx.data.read().await;

        let delete_reply_config_lock = data_read
            .get::<DeleteReplyReactionConfig>()
            .ok_or(BotClientErrors::NoDeleteReply)?
            .clone();
        let emoji = delete_reply_config_lock.read().await.clone();

        Ok(emoji)
    }

    async fn reaction_handler(
        &self,
        ctx: Context,
        reaction_add: Reaction,
    ) -> Result<(), BotClientErrors> {
        let emoji = reaction_add.emoji.clone();
        let reaction_user = reaction_add.user(&ctx).await?;
        let guild = reaction_add
            .guild_id
            .map(|g| g.get().to_string())
            .unwrap_or(String::from("None"));
        self.get_reaction_message(&ctx, reaction_add)
            .and_then(|(ctx, msg, emoji)| async move {
                let author = msg.author.clone();
                let me = ctx.cache.current_user().clone();
                if author.eq(&me) {
                    Ok((ctx, msg, emoji))
                } else {
                    Err(BotClientErrors::NotMyMessage)
                }
            })
            .and_then(|(ctx, msg, delete_emoji)| async move {
                if emoji.unicode_eq(delete_emoji.as_ref()) {
                    Ok((ctx, msg))
                } else {
                    Err(BotClientErrors::InvalidEmoji)
                }
            })
            .and_then(|(ctx, msg)| async move {
                future::ready(
                    msg.referenced_message
                        .clone()
                        .ok_or(BotClientErrors::NoReply),
                )
                .and_then(|ref_msg| async move {
                    let user_id = ref_msg.author.id.clone();
                    if ref_msg.author.eq(&reaction_user) {
                        Ok(user_id)
                    } else {
                        Err(BotClientErrors::NotOriginalAuthor)
                    }
                })
                .and_then(|user| async move {
                    info! {%guild, %user, "deleting reply to user"};
                    msg.delete(&ctx).map_err(Self::parse_errors).await
                })
                .await
            })
            .await
    }

    async fn process_message<'a>(
        &self,
        ctx: &'a Context,
        message: Message,
    ) -> Result<(&'a Context, Message, String), BotClientErrors> {
        let data_read = ctx.data.read().await;

        let msg_processor_lock = data_read
            .get::<MessageHandler>()
            .ok_or(BotClientErrors::NoProcessors)?
            .clone();

        let processor = msg_processor_lock.read().await;

        processor
            .process_message(&message.content)
            .ok_or(BotClientErrors::NotModified)
            .map(|reply| (ctx, message, reply))
    }

    fn parse_errors(err: SerenityError) -> BotClientErrors {
        match err {
            SerenityError::Model(ModelError::InvalidPermissions { required, present }) => {
                let difference = required.difference(present);
                if difference.contains(Permissions::SEND_MESSAGES)
                    || difference.contains(Permissions::SEND_MESSAGES_IN_THREADS)
                {
                    BotClientErrors::InsufficientPermissions(NeededPermissions::SendMessage)
                } else {
                    err.into()
                }
            }
            SerenityError::Http(HttpError::UnsuccessfulRequest(ErrorResponse {
                status_code: StatusCode::FORBIDDEN,
                ..
            })) => BotClientErrors::InsufficientPermissions(NeededPermissions::EditMessage),
            e => e.into(),
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    #[instrument(skip(self, ctx, _ready))]
    async fn ready(&self, ctx: Context, _ready: Ready) {
        {
            let mut data = ctx.data.write().await;
            let status = data.get_mut::<BotState>().expect("could not get bot state");
            status.store(
                crate::BotStatus::Ready,
                std::sync::atomic::Ordering::Relaxed,
            );
        }
        info!("Link Pedant ready!");
        let invite_url =
            get_invite_command(ctx.http.application_id().expect("no application id is set")).await;
        info!("You can invite the bot using this url: {invite_url}");
        let cmds = LinkPedantCommands::create_commands();
        let _ = Command::set_global_commands(&ctx.http, cmds.clone())
            .await
            .map_err(|err| warn! {%err, "could not create global commands"});
        for guild in &ctx.cache.guilds() {
            let _ = guild
                .set_commands(&ctx.http, cmds.clone())
                .await
                .map_err(|err| warn! {%guild, %err, "error setting guild commands"});
        }
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
            let delete_emoji = self
                .get_reaction_emoji(&ctx)
                .await
                .expect("could not get reaction emoji");
            command
                .create_response(
                    ctx,
                    CreateInteractionResponse::Message(
                        CreateInteractionResponseMessage::new()
                            .ephemeral(true)
                            .content(
                                command_data
                                    .run(client_id, delete_emoji.as_ref(), &command.locale)
                                    .await,
                            ),
                    ),
                )
                .await
                .map_err(|err| warn! {%err, "could not send response"})
                .unwrap();
        }
    }

    #[instrument(skip(self, ctx, message))]
    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            debug!("message is from a bot, ignoring...");
            return;
        }

        if let Err(err) = self.message_handler(ctx, message).await {
            if let BotClientErrors::InsufficientPermissions(NeededPermissions::SendMessage) = err {
                info!("cannot reply to message, ignoring...");
            } else {
                warn! {%err, "processing message"};
            }
        }
    }

    #[instrument(skip(self, ctx, reaction_add))]
    async fn reaction_add(&self, ctx: Context, reaction_add: Reaction) {
        if let Err(err) = self.reaction_handler(ctx, reaction_add).await {
            match err {
                BotClientErrors::NotMyMessage => debug!("reaction was not on my message"),
                BotClientErrors::InvalidEmoji => debug!("reaction was not the correct emoji"),
                BotClientErrors::NotOriginalAuthor => {
                    debug!("reaction wasn't from original author")
                }
                err => {
                    warn! {%err, "handling reaction add"}
                }
            }
        }
    }
}
