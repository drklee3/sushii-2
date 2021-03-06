use serenity::framework::standard::macros::hook;
use serenity::framework::standard::CommandError;
use serenity::framework::standard::DispatchError;
use serenity::model::prelude::*;
use serenity::prelude::*;

pub mod normal_message;
pub use self::normal_message::normal_message;

use crate::model::sql::GuildConfig;

#[hook]
pub async fn before(ctx: &Context, msg: &Message, cmd_name: &str) -> bool {
    let guild_conf = match GuildConfig::from_msg(&ctx, &msg).await.map_err(|e| {
        tracing::warn!("Failed to get guild config: {}", e);
    }) {
        Ok(Some(c)) => c,
        Ok(None) => return true, // in dms
        _ => return false,       // something failed
    };

    if let Some(channel) = guild_conf.role_channel {
        if msg.channel_id == channel as u64 {
            tracing::debug!(?msg, "Skipped command in role channel");
            return false;
        }
    }

    if let Some(disabled_channels) = guild_conf.disabled_channels {
        if disabled_channels.contains(&(msg.channel_id.0 as i64)) {
            return false;
        }
    }

    tracing::info!(author = %msg.author.tag(), %msg.content, "Running command {}", cmd_name);

    true
}

#[hook]
pub async fn dispatch_error(ctx: &Context, msg: &Message, error: DispatchError) {
    match error {
        DispatchError::NotEnoughArguments { min, given } => {
            let s = format!("This command needs {} arguments, only got {}.", min, given);

            let _ = msg.channel_id.say(&ctx, &s).await;
        }
        DispatchError::TooManyArguments { max, given } => {
            let s = format!("Max arguments allowed is {}, but got {}.", max, given);

            let _ = msg.channel_id.say(&ctx, &s).await;
        }
        DispatchError::LackingPermissions(permissions) => {
            let s = format!(
                "You do not have permissions to use this command, requires: `{}`",
                permissions
            );

            let _ = msg.channel_id.say(&ctx, &s).await;
        }
        _ => tracing::warn!("Unhandled dispatch error: {:?}", error),
    }
}

#[hook]
pub async fn after(ctx: &Context, msg: &Message, cmd_name: &str, error: Result<(), CommandError>) {
    // Errors here are only from sushii errors, not user input errors
    if let Err(e) = error {
        tracing::error!(?msg, %e, "Error running command");

        // Downcast error
        sentry::capture_error(&*e);

        let _ = msg
            .channel_id
            .say(
                &ctx,
                format!("Something went wrong while running this command :(\n{}", e),
            )
            .await;
    }

    metrics::counter!("commands", 1, "command_name" => cmd_name.to_string());
}
