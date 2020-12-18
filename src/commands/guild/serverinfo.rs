use num_format::{Locale, ToFormattedString};
use serenity::framework::standard::{macros::command, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::fmt::Write;

#[command]
#[aliases("guildinfo")]
#[only_in("guild")]
async fn serverinfo(ctx: &Context, msg: &Message) -> CommandResult {
    let guild = match msg.guild(ctx).await {
        Some(id) => id,
        None => {
            msg.channel_id.say(&ctx.http, "No guild found").await?;

            return Ok(());
        }
    };

    let owner = guild.owner_id.to_user(ctx).await?;

    let mut guild_str = String::new();

    writeln!(guild_str, "**Owner:** {} (ID {})", owner.tag(), owner.id.0)?;
    writeln!(guild_str, "**Created:** {}", guild.id.created_at().format("%Y-%m-%d %H:%M:%S"))?;
    writeln!(guild_str, "**Members:** {}", guild.member_count.to_formatted_string(&Locale::en))?;
    writeln!(guild_str, "**Region:** {}", guild.region)?;
    writeln!(guild_str, "**Roles:** {}", guild.roles.len())?;
    writeln!(guild_str, "**Verification Level:** {:?}", guild.verification_level)?;
    writeln!(guild_str, "**Explicit Content Filter:** {:?}", guild.explicit_content_filter)?;

    if !guild.features.is_empty() {
        writeln!(guild_str, "**Features:** {}", guild.features.join(", "))?;
    }

    let (text_channels, voice_channels) = guild.channels.values().fold((0, 0), |mut acc, chan| {
        if chan.kind == ChannelType::Text {
            acc.0 += 1;
        } else if chan.kind == ChannelType::Voice {
            acc.1 += 1;
        };

        acc
    });

    writeln!(guild_str, "**Channels:** {} text, {} voice", text_channels, voice_channels)?;

    let (emojis, animated_emojis) = guild.emojis.values().fold((0, 0), |mut acc, emoji| {
        if emoji.animated {
            acc.1 += 1;
        } else {
            acc.0 += 1;
        }

        acc
    });

    writeln!(guild_str, "**Emojis:** {} static, {} animated", emojis, animated_emojis)?;

    msg.channel_id
        .send_message(ctx, |m| {
            m.embed(|e| {
                e.author(|a| {
                    a.name(&guild.name);

                    if let Some(url) = guild.icon_url() {
                        a.url(url);
                    }

                    a
                });

                if let Some(url) = guild.icon_url() {
                    e.thumbnail(url);
                }

                e.description(guild_str);

                e.footer(|f| {
                    f.text(&format!("Guild ID: {}", &guild.id.0));

                    f
                });

                e
            })
        })
        .await?;

    Ok(())
}