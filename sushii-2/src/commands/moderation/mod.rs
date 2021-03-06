use serenity::framework::standard::macros::group;

mod cases;
mod channel;
mod chat;

use self::{
    cases::{ban::*, delete::*, history::*, kick::*, mute::*, reason::*, warn::*},
    channel::*,
    chat::*,
};

#[group]
#[commands(
    prune, history, ban, unban, kick, mute, listmutes, reason, unmute, warn, slowmode, deletecase
)]
#[only_in("guild")]
#[required_permissions("BAN_MEMBERS")]
pub struct Moderation;
