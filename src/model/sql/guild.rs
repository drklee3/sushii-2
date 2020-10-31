use serde::{Deserialize, Serialize};
use serenity::async_trait;
use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::parse_channel;

use super::GuildSetting;
use crate::prelude::*;

#[async_trait]
pub trait GuildConfigDb {
    async fn from_msg(ctx: &Context, msg: &Message) -> Result<Option<GuildConfig>>;
    async fn from_msg_or_respond(ctx: &Context, msg: &Message) -> Result<GuildConfig>;
    async fn from_id(ctx: &Context, guild_id: &GuildId) -> Result<Option<GuildConfig>>;

    async fn get(
        ctx: &Context,
        msg: Option<&Message>,
        guild_id: Option<&GuildId>,
    ) -> Result<Option<GuildConfig>>;

    async fn cache(&self, ctx: &Context);
    async fn save(&self, ctx: &Context) -> Result<()>;
    async fn save_db(&self, ctx: &Context) -> Result<()>;
    fn save_bg(&self, ctx: &Context);
}

#[derive(Deserialize, Default, Serialize, sqlx::FromRow, Clone, Debug)]
pub struct GuildConfig {
    pub id: i64,
    pub name: Option<String>,
    pub prefix: Option<String>,

    /// Join message text
    pub join_msg: Option<String>,
    pub join_msg_enabled: bool,

    /// Join message reaction
    pub join_react: Option<String>,

    /// Leave message text
    pub leave_msg: Option<String>,
    pub leave_msg_enabled: bool,

    /// Join / leave messages channel
    pub msg_channel: Option<i64>,

    /// Role assignments
    pub role_channel: Option<i64>,
    pub role_config: Option<serde_json::Value>,
    pub role_enabled: bool,

    /// Auto delete invite links
    pub invite_guard: bool,

    /// Message deleted / edited log channel
    pub log_msg: Option<i64>,
    pub log_msg_enabled: bool,

    /// Moderation actions log channel
    pub log_mod: Option<i64>,
    pub log_mod_enabled: bool,

    /// Member join / leave log channel
    pub log_member: Option<i64>,

    /// Mute role ID
    pub mute_role: Option<i64>,
    /// Duration in seconds
    pub mute_duration: Option<i64>,

    /// Should DM user on ban
    pub ban_dm_text: Option<String>,
    pub ban_dm_enabled: bool,

    /// Should DM user on kick
    pub kick_dm_text: Option<String>,
    pub kick_dm_enabled: bool,

    /// Should DM user on mute
    pub mute_dm_text: Option<String>,
    pub mute_dm_enabled: bool,

    /// Max number of unique mentions in a single message to auto mute
    pub max_mention: Option<i32>,
}

impl GuildConfig {
    pub fn new(id: i64) -> Self {
        GuildConfig {
            id,
            // Toggles should default to true since it still requires the main
            // setting to be set, if setting is set then it should mean they want
            // it to be turned on then.
            join_msg_enabled: true,
            leave_msg_enabled: true,
            log_msg_enabled: true,
            log_mod_enabled: true,
            mute_dm_enabled: true,
            ..Default::default()
        }
    }

    pub fn set_setting(&mut self, setting: &GuildSetting, val: &str) -> Result<()> {
        match setting {
            GuildSetting::JoinMsg => {
                self.join_msg.replace(val.into());
            }
            GuildSetting::JoinReact => {
                self.join_react.replace(val.into());
            }
            GuildSetting::LeaveMsg => {
                self.leave_msg.replace(val.into());
            }
            GuildSetting::MsgChannel => {
                self.msg_channel.replace(
                    parse_channel(val).ok_or_else(|| Error::Sushii("invalid channel".into()))?
                        as i64,
                );
            }
            GuildSetting::MsgLog => {
                self.log_msg.replace(
                    parse_channel(val).ok_or_else(|| Error::Sushii("invalid channel".into()))?
                        as i64,
                );
            }
            GuildSetting::ModLog => {
                self.log_mod.replace(
                    parse_channel(val).ok_or_else(|| Error::Sushii("invalid channel".into()))?
                        as i64,
                );
            }
            GuildSetting::MemberLog => {
                self.log_member.replace(
                    parse_channel(val).ok_or_else(|| Error::Sushii("invalid channel".into()))?
                        as i64,
                );
            }
            GuildSetting::MuteDm => {
                self.mute_dm_text.replace(val.into());
            }
        }

        Ok(())
    }

    /// Enables/Disables a given setting, returns Ok(true) if successfully changed,
    /// Ok(false) if it is already enabled and Err() if the setting cannot be
    /// enabled or disabled
    fn update_setting(&mut self, setting: &GuildSetting, new_value: bool) -> Result<bool> {
        match setting {
            GuildSetting::JoinMsg => {
                if self.join_msg_enabled == new_value {
                    return Ok(false);
                }

                self.join_msg_enabled = new_value;
            }
            GuildSetting::LeaveMsg => {
                if self.leave_msg_enabled == new_value {
                    return Ok(false);
                }

                self.leave_msg_enabled = new_value;
            }
            GuildSetting::MsgLog => {
                if self.log_msg_enabled == new_value {
                    return Ok(false);
                }

                self.log_msg_enabled = new_value;
            }
            GuildSetting::ModLog => {
                if self.log_mod_enabled == new_value {
                    return Ok(false);
                }

                self.log_mod_enabled = new_value;
            }
            GuildSetting::MuteDm => {
                if self.mute_dm_enabled == new_value {
                    return Ok(false);
                }

                self.mute_dm_enabled = new_value;
            }
            GuildSetting::JoinReact | GuildSetting::MsgChannel | GuildSetting::MemberLog => {
                return Err(Error::Sushii(
                    "this setting cannot be enabled/disabled".into(),
                ));
            }
        }

        Ok(true)
    }

    pub fn enable_setting(&mut self, setting: &GuildSetting) -> Result<bool> {
        self.update_setting(setting, true)
    }

    pub fn disable_setting(&mut self, setting: &GuildSetting) -> Result<bool> {
        self.update_setting(setting, false)
    }

    /// Toggles a setting and returns it's new value
    pub fn toggle_setting(&mut self, setting: &GuildSetting) -> Result<bool> {
        let new_val = match setting {
            GuildSetting::JoinMsg => {
                self.join_msg_enabled = !self.join_msg_enabled;
                self.join_msg_enabled
            }
            GuildSetting::LeaveMsg => {
                self.leave_msg_enabled = !self.leave_msg_enabled;
                self.leave_msg_enabled
            }
            GuildSetting::MsgLog => {
                self.log_msg_enabled = !self.log_msg_enabled;
                self.log_msg_enabled
            }
            GuildSetting::ModLog => {
                self.log_mod_enabled = !self.log_mod_enabled;
                self.log_mod_enabled
            }
            GuildSetting::MuteDm => {
                self.mute_dm_enabled = !self.mute_dm_enabled;
                self.mute_dm_enabled
            }
            GuildSetting::JoinReact | GuildSetting::MsgChannel | GuildSetting::MemberLog => {
                return Err(Error::Sushii(
                    "this setting cannot be enabled/disabled".into(),
                ));
            }
        };

        Ok(new_val)
    }

    pub fn get_setting(&self, setting: &GuildSetting) -> (Option<String>, Option<bool>) {
        match setting {
            GuildSetting::JoinMsg => (self.join_msg.clone(), Some(self.join_msg_enabled)),
            GuildSetting::LeaveMsg => (self.leave_msg.clone(), Some(self.leave_msg_enabled)),
            GuildSetting::MsgLog => (
                self.log_msg.map(|id| format!("<#{}>", id as u64)),
                Some(self.log_msg_enabled),
            ),
            GuildSetting::ModLog => (
                self.log_mod.map(|id| format!("<#{}>", id as u64)),
                Some(self.log_mod_enabled),
            ),
            GuildSetting::MuteDm => (self.mute_dm_text.clone(), Some(self.mute_dm_enabled)),
            GuildSetting::JoinReact => (self.join_react.clone(), None),
            GuildSetting::MsgChannel => {
                (self.msg_channel.map(|id| format!("<#{}>", id as u64)), None)
            }
            GuildSetting::MemberLog => {
                (self.log_member.map(|id| format!("<#{}>", id as u64)), None)
            }
        }
    }
}

#[async_trait]
impl GuildConfigDb for GuildConfig {
    /// Gets a GuildConfig from a given message
    async fn from_msg(ctx: &Context, msg: &Message) -> Result<Option<GuildConfig>> {
        GuildConfig::get(ctx, Some(msg), None).await
    }

    /// Gets a GuildConfig from a message, and responds in channel if no guild is found
    async fn from_msg_or_respond(ctx: &Context, msg: &Message) -> Result<GuildConfig> {
        match GuildConfig::from_msg(ctx, msg).await? {
            Some(conf) => Ok(conf),
            None => {
                if let Err(e) = msg
                    .channel_id
                    .say(&ctx.http, "Failed to get the guild config :(")
                    .await
                {
                    tracing::error!(?msg, "Failed to send message: {}", e);
                }

                tracing::warn!(?msg, "Failed to get guild config");

                Err(Error::Sushii("Failed to get guild config".into()))
            }
        }
    }

    /// Gets a Guildconfig from a guild id
    async fn from_id(ctx: &Context, guild_id: &GuildId) -> Result<Option<GuildConfig>> {
        GuildConfig::get(ctx, None, Some(guild_id)).await
    }

    /// Gets a GuildConfig from the cache or database
    async fn get(
        ctx: &Context,
        msg: Option<&Message>,
        guild_id: Option<&GuildId>,
    ) -> Result<Option<GuildConfig>> {
        // Return None if no guild found
        let guild_id = match guild_id.or_else(|| msg.and_then(|m| m.guild_id.as_ref())) {
            Some(id) => id,
            None => return Ok(None),
        };

        let data = ctx.data.read().await;
        let sushii_cache = data.get::<SushiiCache>().unwrap();

        // Check if exists in cache
        if sushii_cache.guilds.contains_key(&guild_id) {
            return Ok(sushii_cache
                .guilds
                .get(&guild_id)
                .map(|e| e.value().clone()));
        }

        // Not in cache, fetch from database
        let pool = data.get::<DbPool>().unwrap();
        let conf = match get_guild_config_query(&pool, guild_id.0)
            .await
            .map_err(|e| {
                tracing::error!(
                    ?msg,
                    ?guild_id,
                    "Failed to get guild config from database: {}",
                    e
                );

                e
            })? {
            Some(c) => c,
            None => {
                let new_conf = GuildConfig::new(guild_id.0 as i64);

                if let Err(e) = new_conf.save_db(&ctx).await {
                    tracing::error!("Failed to save new guild config: {}", e);
                }

                new_conf
            }
        };

        conf.cache(&ctx).await;

        Ok(Some(conf))
    }

    /// Updates config in the cache
    async fn cache(&self, ctx: &Context) {
        let data = ctx.data.read().await;
        let sushii_cache = data.get::<SushiiCache>().unwrap();

        sushii_cache
            .guilds
            .insert(GuildId(self.id as u64), self.clone());

        tracing::info!(config = ?self, "Cached guild config");
    }

    /// Saves config to database
    async fn save_db(&self, ctx: &Context) -> Result<()> {
        let data = ctx.data.read().await;
        let pool = data.get::<DbPool>().unwrap();

        // Update db and cache
        upsert_config_query(self, pool).await?;
        Ok(())
    }

    /// Saves config to both database and cache
    async fn save(&self, ctx: &Context) -> Result<()> {
        self.save_db(&ctx).await?;
        self.cache(&ctx).await;

        Ok(())
    }

    /// Saves config in the background, does NOT respond with an error but does log errors
    /// Mainly used because this isn't an async fn, can be used in or_else
    fn save_bg(&self, ctx: &Context) {
        let conf = self.clone();
        let ctx = ctx.clone();
        tokio::spawn(async move {
            if let Err(e) = conf.save(&ctx).await {
                tracing::error!("Failed to save config in background: {}", e);
            }
        });
    }
}

async fn get_guild_config_query(pool: &sqlx::PgPool, guild_id: u64) -> Result<Option<GuildConfig>> {
    sqlx::query_as!(
        GuildConfig,
        r#"
            SELECT *
              FROM guild_configs
             WHERE id = $1
        "#,
        guild_id as i64
    )
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

async fn upsert_config_query(conf: &GuildConfig, pool: &sqlx::PgPool) -> Result<()> {
    // Bruh
    sqlx::query_file!(
        "sql/guild/upsert_guild_config.sql",
        conf.id,
        conf.name,
        conf.prefix,
        conf.join_msg,
        conf.join_msg_enabled,
        conf.join_react,
        conf.leave_msg,
        conf.leave_msg_enabled,
        conf.msg_channel,
        conf.role_channel,
        conf.role_config,
        conf.role_enabled,
        conf.invite_guard,
        conf.log_msg,
        conf.log_msg_enabled,
        conf.log_mod,
        conf.log_mod_enabled,
        conf.log_member,
        conf.mute_role,
        conf.mute_duration,
        conf.ban_dm_text,
        conf.ban_dm_enabled,
        conf.kick_dm_text,
        conf.kick_dm_enabled,
        conf.mute_dm_text,
        conf.mute_dm_enabled,
        conf.max_mention,
    )
    .execute(pool)
    .await
    .map(|_| ())
    .map_err(Into::into)
}
