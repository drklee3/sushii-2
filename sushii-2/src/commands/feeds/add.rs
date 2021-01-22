use futures::stream::StreamExt;
use serenity::async_trait;
use serenity::framework::standard::{macros::command, Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::{parse_channel, parse_role};
use std::result::Result as StdResult;
use vlive::VLiveRequester;
use std::time::Duration;

use crate::error::Result;
use crate::keys::*;
use sushii_model::model::sql::feed::feed::Id;
use sushii_model::model::sql::{Feed, FeedMetadata, FeedSubscription};

#[derive(Default, Debug, Clone)]
struct FeedOptions {
    pub kind: Option<String>,
    pub discord_channel: Option<u64>,
    pub mention_role: Option<u64>,
}

impl FeedOptions {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
pub trait UserOption<T> {
    fn prompt(&self) -> &'static str;
    async fn validate(
        &self,
        ctx: &Context,
        msg: &Message,
        state: &mut T,
    ) -> StdResult<String, String>;
}

pub struct OptionsCollector<T> {
    options: Vec<Box<dyn UserOption<T> + Send + Sync>>,
    /// Item to modify when an option is valid, where to store responses
    /// e.g a struct or hashmap
    state: T,
}

impl<T> OptionsCollector<T> {
    pub fn new(state: T) -> Self {
        Self {
            options: vec![],
            state,
        }
    }

    pub fn add_option(mut self, opt: impl UserOption<T> + 'static + Send + Sync) -> Self {
        self.options.push(Box::new(opt));
        self
    }

    pub fn get_state(&self) -> &T {
        &self.state
    }

    async fn collect(&mut self, ctx: &Context, msg: &Message) -> Result<()> {
        for option in &self.options {
            msg.channel_id.say(ctx, option.prompt()).await?;

            let mut replies = msg
                .channel_id
                .await_replies(ctx)
                .author_id(msg.author.id)
                .channel_id(msg.channel_id)
                .timeout(Duration::from_secs(120))
                .await;

            while let Some(reply) = replies.next().await {
                match option.validate(ctx, &reply, &mut self.state).await {
                    Ok(response) => {
                        msg.channel_id.say(ctx, response).await?;

                        // If success, break from waiting for response and go to
                        // next option
                        break;
                    }
                    Err(response) => {
                        // If option isn't valid, respond with error and wait for another try
                        msg.channel_id
                            .say(ctx, format!("Error: {}", response))
                            .await?;
                    }
                }
            }

            // TODO: Delete messages
        }

        Ok(())
    }
}

struct FeedType;

#[async_trait]
impl UserOption<FeedOptions> for FeedType {
    fn prompt(&self) -> &'static str {
        "What kind of feed do you want to add? Currently available feeds are: `vlive`, `twitter`"
    }

    async fn validate(
        &self,
        _ctx: &Context,
        msg: &Message,
        state: &mut FeedOptions,
    ) -> StdResult<String, String> {
        match msg.content.as_str() {
            "vlive" => {
                state.kind.replace("vlive".into());
                return Ok(format!("Selected feed type {}", msg.content));
            }
            // RSS feeds later
            // "twitter" | "youtube" => "rss",
            _ => {
                return Err(format!(
                    "{} is not a valid feed type. Currently available feeds are: `vlive`",
                    msg.content
                ));
            }
        }
    }
}

struct DiscordChannel;

#[async_trait]
impl UserOption<FeedOptions> for DiscordChannel {
    fn prompt(&self) -> &'static str {
        "Which channel do you want updates to be sent to?"
    }

    async fn validate(
        &self,
        ctx: &Context,
        msg: &Message,
        state: &mut FeedOptions,
    ) -> StdResult<String, String> {
        let channel_id = msg
            .content
            .parse::<u64>()
            .ok()
            .or_else(|| parse_channel(&msg.content))
            .ok_or("Invalid channel. Give a channel.".to_string())?;

        let guild_channels = msg
            .guild_field(ctx, |g| g.channels.clone())
            .await
            .ok_or("Couldn't find channel. Give a channel.".to_string())?;

        match guild_channels.get(&ChannelId(channel_id)) {
            Some(c) => {
                if c.kind != ChannelType::Text {
                    return Err("Channel is not a text channel. Try a different one.".into());
                }

                state.discord_channel.replace(c.id.0);

                return Ok(format!("Updates will be sent to <#{}>", c.id.0));
            }
            None => {
                return Err("Channel is not found in this guild. Try again?".into());
            }
        }
    }
}

struct DiscordRole;

#[async_trait]
impl UserOption<FeedOptions> for DiscordRole {
    fn prompt(&self) -> &'static str {
        "What role do you want to mention for new updates? Say `none` for no role mention."
    }

    async fn validate(
        &self,
        ctx: &Context,
        msg: &Message,
        state: &mut FeedOptions,
    ) -> StdResult<String, String> {
        if msg.content.trim() == "none" {
            return Ok("This feed won't mention any roles.".into());
        }

        // TODO: actually handle this, not an accurate error
        let guild_roles = msg
            .guild_field(ctx, |g| g.roles.clone())
            .await
            .ok_or("Couldn't find role. Give a role.".to_string())?;

        let mention_role = parse_role(&msg.content)
            .or_else(|| msg.content.parse::<u64>().ok())
            .or_else(|| {
                guild_roles
                    .values()
                    .find(|&x| x.name.to_lowercase() == msg.content.to_lowercase())
                    .map(|x| x.id.0)
            })
            .ok_or("Invalid role, give a role name, role mention, or role ID. `none` for no mention role.".to_string())?;

        state.mention_role.replace(mention_role);

        Ok(format!("The role will be mentioned <@&{}>", mention_role))
    }
}

#[command]
#[only_in("guild")]
#[required_permissions("MANAGE_GUILD")]
async fn add(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let guild = match msg.guild(ctx).await {
        Some(g) => g,
        None => {
            msg.reply(&ctx, "Error: No guild").await?;

            return Ok(());
        }
    };

    let reqwest = ctx
        .data
        .read()
        .await
        .get::<ReqwestContainer>()
        .cloned()
        .unwrap();

    let mut messages = msg
        .channel_id
        .await_replies(ctx)
        .author_id(msg.author.id)
        .channel_id(msg.channel_id)
        .await;

    let mut options_collector = OptionsCollector::new(FeedOptions::new())
        .add_option(FeedType)
        .add_option(DiscordChannel)
        .add_option(DiscordRole);

    options_collector.collect(ctx, msg).await?;

    let opts = options_collector.get_state();

    let feed_metadata = match opts.kind.as_ref().unwrap().to_lowercase().as_str() {
        "vlive" => match add_vlive(reqwest, ctx, msg).await? {
            Some(m) => m,
            None => return Ok(()),
        },
        // "twitter" | "twt" => {}
        _ => {
            msg.reply(
                &ctx,
                "Error: Invalid feed type. \
                    Currently available feeds are: `vlive`",
            )
            .await?;

            return Ok(());
        }
    };

    // Need to save the Feed data, or ensure it already exists before saving subscription
    let feed = Feed::from_meta(feed_metadata).save(ctx).await?;

    let subscription = FeedSubscription::new(
        feed.feed_id,
        guild.id.0 as i64,
        opts.discord_channel.unwrap() as i64,
    )
    .mention_role(opts.mention_role.map(|r| r as i64))
    .save(ctx)
    .await?;

    dbg!(subscription);

    Ok(())
}

enum VliveStep {
    Channel,
    Board,
}

impl VliveStep {
    pub fn start() -> Self {
        Self::Channel
    }

    pub fn next(self, is_videos: bool) -> Option<Self> {
        match self {
            Self::Channel if is_videos => None,
            Self::Channel => Some(Self::Board),
            Self::Board => None,
        }
    }
}

// vlive feeds are hardcoded since needs to search channels, handle more stuff etc
async fn add_vlive(
    reqwest: reqwest::Client,
    ctx: &Context,
    msg: &Message,
) -> Result<Option<FeedMetadata>> {
    let mut messages = msg
        .channel_id
        .await_replies(ctx)
        .author_id(msg.author.id)
        .channel_id(msg.channel_id)
        .await;

    let mut step = VliveStep::start();

    msg.channel_id
        .say(
            &ctx,
            "What vlive channel? Give a vlive channel code. You can find this \
            in the channel URL, for example `F001E5` is the code for \
            `https://www.vlive.tv/channel/F001E5` Type `quit` any time to stop.",
        )
        .await?;

    let mut feed_metadata = None;

    while let Some(reply) = messages.next().await {
        match reply.content.as_str() {
            "quit" | "stop" | "exit" => {
                msg.channel_id
                    .say(&ctx, "Quitting. No feeds were added.")
                    .await?;

                return Ok(None);
            }
            _ => {}
        }

        match step {
            VliveStep::Channel => {
                let channel = match reqwest.get_channel_info(&reply.content).await {
                    Ok(c) => c,
                    Err(_) => {
                        msg.reply(
                            &ctx,
                            format!(
                                "Error: No channel was found with code `{}`. \
                                Give a vlive channel code. You can find this \
                                in the channel URL, for example `F001E5` is the code for \
                                `https://www.vlive.tv/channel/F001E5`. Type `quit` any time to stop.",
                                &reply.content
                            ),
                        )
                        .await?;

                        continue;
                    }
                };

                msg.reply(&ctx, format!("Found channel {}", &channel.name))
                    .await?;

                feed_metadata = Some(FeedMetadata::vlive_videos(
                    None,
                    channel.channel_code,
                    channel.name,
                    channel.profile_img,
                ));
            }
            VliveStep::Board => {
                break;
            }
        }

        step = match step.next(true) {
            Some(s) => s,
            None => break,
        };
    }

    Ok(feed_metadata)
}
