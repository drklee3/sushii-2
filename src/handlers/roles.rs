use futures::join;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use serenity::{model::prelude::*, prelude::*};
use std::collections::{HashMap, HashSet};
use std::fmt::Write;
use std::time::Duration;
use std::vec::Vec;
use tokio::time::delay_for;

use crate::error::Result;
use crate::keys::CacheAndHttpContainer;
use crate::model::sql::{
    guild_roles::{GuildRole, GuildRoles},
    GuildConfig, GuildConfigDb,
};

#[derive(Debug, Eq, PartialEq)]
enum RoleActionKind {
    Add,
    Remove,
}

#[derive(Debug)]
struct RoleAction {
    pub index: usize,
    pub kind: RoleActionKind,
    pub role_name: String,
}

fn vec_to_code_string<S: AsRef<str>, I: IntoIterator<Item = S>>(v: I) -> String {
    v.into_iter()
        .map(|s| format!("`{}`", s.as_ref()))
        .collect::<Vec<String>>()
        .join(", ")
}

pub async fn message(ctx: &Context, msg: &Message) {
    match _message(ctx, msg).await {
        Ok(msg_string) => {
            let msg_string = match msg_string {
                Some(s) => s,
                None => return,
            };

            let sent_msg = msg.channel_id.say(&ctx.http, &msg_string).await;

            if let Err(e) = sent_msg.as_ref() {
                tracing::warn!(message=%msg_string, "Failed to send role message: {}", e);
            };

            // Delete messages after 10 seconds
            delay_for(Duration::from_secs(10)).await;

            let data = &ctx.data.read().await;
            let cache_http = data.get::<CacheAndHttpContainer>().unwrap();

            if let Ok(sent_msg) = sent_msg {
                // Run both delete futures concurrently instead of in series
                // try_join! better for Results but still want to try deleting both as
                // try_join! short circuits and returns immediately on any Error
                let (recv_res, sent_res) = join!(msg.delete(&cache_http), sent_msg.delete(&cache_http));

                if let Err(e) = recv_res {
                    tracing::warn!(?msg, "Failed to delete received message: {}", e);
                }

                if let Err(e) = sent_res {
                    tracing::warn!(message=%msg_string, "Failed to delete sent message: {}", e);
                }
            } else {
                // Role message failed sooo just delete user's message
                let _ = msg.delete(&cache_http).await;
            }

        },
        Err(e) => {
            delay_for(Duration::from_secs(5)).await;
            tracing::error!(?msg, "Failed to handle roles message: {}", e);

            let data = &ctx.data.read().await;
            let cache_http = data.get::<CacheAndHttpContainer>().unwrap();

            let _ = msg.delete(&cache_http).await;
        }
    }
}

pub async fn _message(ctx: &Context, msg: &Message) -> Result<Option<String>> {
    // ignore self and bots
    if msg.author.bot {
        return Ok(None);
    }

    let guild = match msg.guild(&ctx.cache).await {
        Some(g) => g,
        None => {
            return Ok(None);
        }
    };

    let guild_conf = match GuildConfig::from_id(&ctx, &guild.id).await? {
        Some(c) => c,
        None => {
            tracing::error!(?msg, "No guild config found while handling roles");
            return Ok(None);
        }
    };

    // get configs
    let role_config: GuildRoles = match guild_conf.role_config {
        Some(c) => serde_json::from_value(c)?,
        None => return Ok(None),
    };

    let role_channel = match guild_conf.role_channel {
        Some(c) => c,
        None => return Ok(None),
    };

    // check if in correct channel
    if msg.channel_id.0 != role_channel as u64 {
        return Ok(None);
    }

    // searching for multiple role assignments
    lazy_static! {
        static ref RE: Regex = RegexBuilder::new(r"(-|\+)([\w ]+)")
            .case_insensitive(true)
            .build()
            .unwrap();
    }

    let data = &ctx.data.read().await;
    let cache_http = data.get::<CacheAndHttpContainer>().unwrap();

    if !RE.is_match(&msg.content) || msg.content != "clear" || msg.content != "reset" {
        return Ok(Some("You can add a role with `+role name` or remove a role with `-role name`.  Use `clear` or `reset` to remove all roles".into()))
    }

    // Vec<("+/-", "role name")
    let role_actions: Vec<RoleAction> = RE
        .captures_iter(&msg.content)
        .enumerate()
        .map(|(index, caps)| {
            // Should be safe to unwrap since both groups are required
            let kind = if caps.get(1).map(|m| m.as_str()).unwrap() == "+" {
                RoleActionKind::Add
            } else {
                RoleActionKind::Remove
            };
            let role_name = caps.get(2).map(|m| m.as_str().trim().to_lowercase()).unwrap();

            RoleAction {
                index,
                kind,
                role_name,
            }
        })
        .collect();

    // Should remove all roles
    let is_reset = msg.content == "clear" || msg.content == "reset";

    let member = guild.member(&cache_http, msg.author.id).await?;

    // All roles of the member, this is modified on role add / remove
    let mut member_all_roles: HashSet<u64> = member.roles.iter().map(|x| x.0).collect();

    // Member roles that are in the role config <Category, Vec<role>>
    let mut member_config_roles: HashMap<&str, HashSet<u64>> = HashMap::new();

    // add the member's current roles
    for (group_name, group) in &role_config.groups {
        let group_entry = member_config_roles
            .entry(group_name)
            .or_insert(HashSet::new());

        for (_role_name, role) in &group.roles {
            if member_all_roles.contains(&role.primary_id) {
                group_entry.insert(role.primary_id);
            }

            if let Some(secondary_id) = role.secondary_id {
                if member_all_roles.contains(&secondary_id) {
                    group_entry.insert(secondary_id);
                }
            }

            // remove role if removing all
            if is_reset {
                if member_all_roles.contains(&role.primary_id) {
                    member_all_roles.remove(&role.primary_id);
                }

                if let Some(id) = role.secondary_id {
                    if member_all_roles.contains(&id) {
                        member_all_roles.remove(&id);
                    }
                }
            }
        }
    }

    // Config roles: map from role name -> (role, group_name)
    let mut role_name_map: HashMap<String, (&GuildRole, &str)> = HashMap::new();
    for (group_name, group) in &role_config.groups {
        for (role_name, role) in &group.roles {
            role_name_map.insert(role_name.trim().to_lowercase(), (&role, &group_name));
        }
    }

    tracing::info!(?role_actions, "role_actions");

    // Not the actual "dedupe" but more to check if a user is adding/removing a
    // same role.
    let mut role_actions_deduped_map: HashMap<String, RoleAction> = HashMap::new();

    for action in role_actions {
        let opposite_kind = if action.kind == RoleActionKind::Add {
            RoleActionKind::Remove
        } else {
            RoleActionKind::Add
        };

        // Remove the opposite action for same roles
        // eg: If there's already a +role1, adding a -role1 will remove the +role1 and vice versa
        // If there's a +role1, adding +role1 will just keep the first one
        if let Some(val) = role_actions_deduped_map.get(&action.role_name) {
            tracing::info!(?action, ?val, "Found duplicate role");

            // Remove existing if there's an opposite
            if val.kind == opposite_kind {
                role_actions_deduped_map.remove(&action.role_name);
            }

            // Don't do anything if there's a same
            continue;
        }

        // No existing same role, now can add it
        role_actions_deduped_map.insert(action.role_name.clone(), action);
    }

    let mut role_actions_deduped: Vec<&RoleAction> = role_actions_deduped_map.values().collect();
    role_actions_deduped.sort_by(|a, b| a.index.cmp(&b.index));

    tracing::info!(?role_actions_deduped, "role_actions_deduped");

    let mut added_role_names = Vec::new();
    let mut removed_role_names = Vec::new();

    let mut added_existing_roles = Vec::new();
    let mut removed_missing_roles = Vec::new();
    let mut over_limit_roles = HashMap::new();

    for action in role_actions_deduped {
        if let Some((role, group_name)) = role_name_map.get(action.role_name.trim()) {
            // Member's current roles in this group
            let cur_group_roles = member_config_roles
                .entry(group_name)
                .or_insert(HashSet::new());

            let conf_group = role_config.groups.get(*group_name).unwrap();

            if action.kind == RoleActionKind::Add {
                // If member already has it
                if cur_group_roles.contains(&role.primary_id) {
                    added_existing_roles.push(&action.role_name[..]);

                    continue;
                }

                // Check limits and if primary or secondary
                if cur_group_roles.len() >= conf_group.limit as usize {
                    let entry = over_limit_roles.entry(group_name.clone()).or_insert(Vec::new());
                    entry.push(action.role_name.clone());

                    /*
                    let _ = writeln!(
                        errors_str,
                        "Cannot add `{}`, role group {} has limit of {} roles",
                        action.role_name, group_name, conf_group.limit
                    );
                    */

                    continue;
                }

                // Add role
                let id_to_add = if cur_group_roles.is_empty() || role.secondary_id.is_none() {
                    role.primary_id
                } else if let Some(id) = role.secondary_id {
                    id
                } else {
                    // Shouldn't reach here but ok just skip
                    continue;
                };

                // This is just to keep track of roles and limits
                cur_group_roles.insert(id_to_add);
                // This is a set to actually send to Discord API of **all**
                // the user's role to update to, as this may include roles
                // not in the role config
                member_all_roles.insert(id_to_add);
                added_role_names.push(&action.role_name[..]);
            } else {
                let has_role = cur_group_roles.contains(&role.primary_id)
                    || role
                        .secondary_id
                        .map_or(false, |id| cur_group_roles.contains(&id));

                if !has_role {
                    removed_missing_roles.push(&action.role_name[..]);

                    continue;
                }

                cur_group_roles.remove(&role.primary_id);
                member_all_roles.remove(&role.primary_id);

                if let Some(secondary_id) = role.secondary_id {
                    cur_group_roles.remove(&secondary_id);
                    member_all_roles.remove(&secondary_id);
                }

                removed_role_names.push(&action.role_name);
            }
        }
    }

    let mut s = String::new();

    if is_reset {
        return Ok(Some("Your roles have been reset.".into()));
    } else if added_role_names.is_empty() && removed_role_names.is_empty()
    {
        return Ok(Some("Couldn't modify your roles".into()));
    }

    if !added_role_names.is_empty() {
        let _ = writeln!(
            s,
            "Added roles: {}",
            vec_to_code_string(added_role_names)
        );
    }

    if !removed_role_names.is_empty() {
        let _ = writeln!(
            s,
            "Removed roles: {}",
            vec_to_code_string(removed_role_names)
        );
    }

    // Check if there are over limit roles
    if !over_limit_roles.is_empty() {
        let _ = write!(
            s,
            "Cannot add roles that exceed role group limits: ",
        );
    }

    for (group_name, role_names) in &over_limit_roles {
        if let Some(group) = &role_config.groups.get(&group_name[..]) {
            let _ = write!(
                s,
                "{} ({} group has a limit of {} roles)",
                vec_to_code_string(role_names),
                group_name,
                group.limit
            );
        }
    }
    // End over limit roles

    if let Err(e) = guild
        .edit_member(&ctx.http, msg.author.id, |m| {
            m.roles(
                &member_all_roles
                    .iter()
                    .map(|i| RoleId(*i))
                    .collect::<Vec<RoleId>>(),
            )
        })
        .await {
            msg.channel_id.say(&ctx.http, "Failed to modify your roles :(").await?;
            tracing::warn!(?msg, "Failed to edit member: {}", e);
        }

    Ok(Some(s))
}
