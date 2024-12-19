use rust_i18n::t;
use serenity::all::{ApplicationId, Permissions};
use serenity_commands::Commands;
use tokio::sync::OnceCell;

static PERMISSIONS: OnceCell<Permissions> = OnceCell::const_new();

#[derive(Debug, Commands)]
pub(crate) enum LinkPedantCommands {
    /// Helpful information from LinkPedant!
    Help,
    /// Invite Link Pedant to your server!
    Invite,
}

impl LinkPedantCommands {
    pub async fn run(self, client_id: ApplicationId) -> String {
        match self {
            Self::Help => t!("help.content").to_string(),
            Self::Invite => {
                let permissions = PERMISSIONS
                    .get_or_init(|| async {
                        Permissions::VIEW_CHANNEL
                            | Permissions::SEND_MESSAGES
                            | Permissions::SEND_MESSAGES_IN_THREADS
                            | Permissions::MANAGE_MESSAGES
                            | Permissions::EMBED_LINKS
                    })
                    .await
                    .bits();
                let invite_url = format!("https://discord.com/oauth2/authorize?client_id={client_id}&scope=bot%20applications.commands&permissions={permissions}");
                t!("invite.content", invite_url = invite_url).to_string()
            }
        }
    }
}
