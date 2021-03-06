use juniper::{graphql_object, FieldResult};
use sushii_model::{
    cursor::encode_cursor,
    model::{
        juniper::Context,
        sql::{CachedGuild, UserLevel, UserLevelRanked, UserXP},
        user::TimeFrame,
        BigInt,
    },
    Error,
};

use crate::{relay::PageInfo, relay_connection};

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    fn apiVersion() -> &str {
        "1.0"
    }

    async fn level(
        ctx: &Context,
        user_id: BigInt,
        guild_id: BigInt,
    ) -> FieldResult<Option<UserLevel>> {
        let user_level = UserLevel::from_id(&ctx.pool, user_id, guild_id).await?;

        Ok(user_level)
    }

    async fn rank(
        ctx: &Context,
        user_id: BigInt,
        guild_id: BigInt,
    ) -> FieldResult<Option<UserLevelRanked>> {
        let user_level_ranked = UserLevelRanked::from_id(&ctx.pool, user_id, guild_id).await?;

        Ok(user_level_ranked)
    }

    /// Get a guild's cached info
    async fn guild(ctx: &Context, guild_id: BigInt) -> FieldResult<Option<CachedGuild>> {
        CachedGuild::from_id(&ctx.pool, guild_id)
            .await
            .map_err(Into::into)
    }

    async fn user_xp_leaderboard_connection(
        ctx: &Context,
        guild_id: Option<BigInt>,
        timeframe: TimeFrame,
        first: BigInt,
        after: Option<String>,
    ) -> FieldResult<UserXPConnection> {
        // Fetch 1 extra to see if theres a next page truncated later
        let first_with_peek = BigInt(first.0 + 1);

        // If None, global ranks
        let (total_count, users) = if let Some(guild_id) = guild_id {
            UserXP::guild_top(&ctx.pool, guild_id, timeframe, first_with_peek, after).await?
        } else {
            UserXP::global_top(&ctx.pool, timeframe, first_with_peek, after).await?
        };

        let users_with_peek_len = users.len();

        let edges: Vec<UserXPEdge> = users
            .into_iter()
            .enumerate()
            // Remove the last one if there is an extra peek element
            .filter(|(i, _)| *i != first_with_peek.0 as usize - 1)
            .map(|(_, node)| {
                // Cursor [XP, user_id] bytes to base64
                let cursor = encode_cursor(node.xp.0, node.user_id.0);

                UserXPEdge { node, cursor }
            })
            .collect();

        let page_info = PageInfo {
            // No backwards pagination support for now
            has_previous_page: false,
            // If can fetch 1 extra then there's a next page, if it's <= then there's not a next page
            has_next_page: first_with_peek.0 as usize == users_with_peek_len,
            start_cursor: edges
                .first()
                .map(|e| e.cursor.clone())
                .ok_or_else(|| Error::Sushii("No data was returned".into()))?,
            end_cursor: edges
                .last()
                .map(|e| e.cursor.clone())
                .ok_or_else(|| Error::Sushii("No data was returned".into()))?,
        };

        Ok(UserXPConnection {
            total_count,
            edges,
            page_info,
        })
    }
}

relay_connection!(UserXPConnection, UserXPEdge, UserXP, Context);
