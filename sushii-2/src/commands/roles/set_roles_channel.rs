use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::parse_channel;

use crate::model::sql::*;

#[command]
#[required_permissions("MANAGE_GUILD")]
#[num_args(1)]
async fn setchannel(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let mut conf = GuildConfig::from_msg_or_respond(&ctx, msg).await?;

    let channel_id = match args.single::<String>().ok().and_then(parse_channel) {
        Some(id) => id,
        None => {
            msg.channel_id
                .say(
                    &ctx.http,
                    "Invalid channel, please provide a guild #channel",
                )
                .await?;

            return Ok(());
        }
    };

    conf.role_channel.replace(channel_id as i64);
    conf.save(&ctx).await?;

    msg.channel_id
        .say(
            &ctx.http,
            format!("Updated roles channel to <#{}>", channel_id),
        )
        .await?;

    Ok(())
}
