use rust_i18n::t;
use serenity::all::{
    ApplicationId, CommandData, CreateBotAuthParameters, CreateCommand, Permissions, Scope,
};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use tokio::sync::OnceCell;
use tracing::debug;

static PERMISSIONS: OnceCell<Permissions> = OnceCell::const_new();
static SCOPES: OnceCell<Vec<Scope>> = OnceCell::const_new();

#[derive(Debug, EnumIter, EnumString, Display, PartialEq)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum LinkPedantCommands {
    Help,
    Invite,
}

impl LinkPedantCommands {
    pub fn create_commands() -> Vec<CreateCommand> {
        let locales: Vec<&'static str> = rust_i18n::available_locales!();
        let locales_iter = locales.into_iter().filter(|l| *l != "en").to_owned();
        let mut create_cmds = Vec::new();
        for cmd_type in Self::iter() {
            debug! {%cmd_type, "create command type"};
            let cmd_name = cmd_type.to_string();
            let description_i18n_str = format!("{cmd_name}.description");
            let available_locals = locales_iter.clone();
            let description_str = t!(description_i18n_str);
            debug! {%description_str, "create command description"}
            let mut new_cmd = CreateCommand::new(cmd_name).description(description_str);
            for locale in available_locals {
                new_cmd = new_cmd
                    .description_localized(locale, t!(description_i18n_str, locale = locale));
            }
            create_cmds.push(new_cmd);
        }
        create_cmds
    }

    pub fn from_command_data(data: &CommandData) -> Result<Self, strum::ParseError> {
        LinkPedantCommands::try_from(data.name.as_str())
    }
}

pub(crate) async fn get_invite_command(client_id: ApplicationId) -> String {
    let permissions = PERMISSIONS
        .get_or_init(|| async {
            Permissions::VIEW_CHANNEL
                | Permissions::SEND_MESSAGES
                | Permissions::SEND_MESSAGES_IN_THREADS
                | Permissions::MANAGE_MESSAGES
                | Permissions::EMBED_LINKS
        })
        .await;
    let scopes = SCOPES
        .get_or_init(|| async { vec![Scope::Bot, Scope::ApplicationsCommands] })
        .await;

    CreateBotAuthParameters::new()
        .client_id(client_id)
        .permissions(*permissions)
        .scopes(scopes)
        .build()
}

impl LinkPedantCommands {
    pub async fn run(self, client_id: ApplicationId, locale: &str) -> String {
        match self {
            Self::Help => {
                let mut other_cmd_descriptions: Vec<String> = Vec::new();
                for cmd in Self::iter()
                    .filter(|t| *t != Self::Help)
                    .map(|c| c.to_string())
                {
                    let translation_str = format!("{cmd}.description");
                    let translated_description = t!(translation_str, locale = locale);
                    other_cmd_descriptions.push(format!("\t/{cmd}\t{translated_description}"));
                }
                let cmd_descriptions = other_cmd_descriptions.join("\n");
                t!(
                    "help.content",
                    command_descriptions = cmd_descriptions,
                    locale = locale
                )
                .to_string()
            }
            Self::Invite => {
                let invite_url = get_invite_command(client_id).await;
                t!("invite.content", invite_url = invite_url, locale = locale).to_string()
            }
        }
    }
}
