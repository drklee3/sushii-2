use chrono::{naive::NaiveDateTime, offset::Utc, Duration};
use rand::distributions::{Bernoulli, Distribution};
use rand::prelude::*;
use rand_distr::StandardNormal;
use serde::{Deserialize, Serialize};
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::error::Result;
use crate::keys::DbPool;

#[derive(Deserialize, Serialize, sqlx::FromRow, Clone, Debug, Default)]
pub struct UserData {
    pub id: i64,
    pub is_patron: bool,
    pub patron_emoji: Option<String>,
    pub rep: i64,
    pub fishies: i64,
    pub last_rep: Option<NaiveDateTime>,
    pub last_fishies: Option<NaiveDateTime>,
    pub profile_data: Option<serde_json::Value>,
    pub lastfm_username: Option<String>,
}

fn eligible(last_time: Option<NaiveDateTime>, cooldown: Duration) -> bool {
    // If None, it's a new user so they're default eligible
    let last_time = match last_time {
        Some(t) => t,
        None => return true,
    };

    let now = Utc::now().naive_utc();

    // Now is past last time + cooldown duration
    now > (last_time + cooldown)
}

fn humantime_remaining(last_time: Option<NaiveDateTime>, cooldown: Duration) -> Option<String> {
    if eligible(last_time, cooldown) {
        return None;
    }

    // Should be Some(t) now since eligible returns true if last_time is None
    let last_time = match last_time {
        Some(t) => t,
        None => return None,
    };

    let now = Utc::now().naive_utc();
    let next_time = last_time + cooldown;

    // Get duration, and then round to nearest second
    let duration_left = next_time - now;
    let duration_left_secs = Duration::seconds(duration_left.num_seconds());

    Some(humantime::format_duration(duration_left_secs.to_std().unwrap()).to_string())
}

impl UserData {
    pub fn new(user_id: UserId) -> Self {
        Self {
            id: user_id.into(),
            ..Default::default()
        }
    }

    /// Returns None if user can rep again, otherwise an Option<String> with time left
    pub fn rep_humantime_cooldown(&self) -> Option<String> {
        humantime_remaining(self.last_rep, Duration::hours(12))
    }

    pub fn fishies_humantime_cooldown(&self) -> Option<String> {
        humantime_remaining(self.last_fishies, Duration::hours(12))
    }

    pub fn reset_last_rep(mut self) -> Self {
        self.last_rep.replace(Utc::now().naive_utc());
        self
    }

    pub fn reset_last_fishy(mut self) -> Self {
        self.last_fishies.replace(Utc::now().naive_utc());
        self
    }

    pub fn inc_rep(mut self) -> Self {
        self.rep += 1;
        self
    }

    pub fn inc_fishies(&mut self, is_self: bool) -> (i64, bool) {
        // 1% chance of golden fishy
        let d = Bernoulli::new(0.01).unwrap();
        let is_golden = d.sample(&mut thread_rng());

        // N(0, 1)
        let mut fishies: f64 = thread_rng().sample(StandardNormal);

        fishies = fishies.abs() * 8.0;
        fishies += 5.0;

        // For someone else, multiply by 1.5
        if !is_self {
            fishies *= 1.7f64;
        }

        // If golden fishy, multiply x6
        if is_golden {
            fishies *= 8.0;
        }

        self.fishies += fishies.round() as i64;

        (fishies.round() as i64, is_golden)
    }

    pub fn rep_level(&self) -> String {
        let num = match self.rep {
            n if n >= 2000 => 11,
            n if n >= 1000 => 10,
            n if n >= 100 => 9,
            n if n >= 50 => 8,
            n if n >= 0 => 7,
            _ => 1,
        };

        format!("{:02}", num)
    }

    pub async fn from_id(ctx: &Context, user_id: UserId) -> Result<Option<UserData>> {
        let pool = ctx.data.read().await.get::<DbPool>().cloned().unwrap();

        from_id_query(&pool, user_id).await
    }

    pub async fn from_id_or_new(ctx: &Context, user_id: UserId) -> Result<UserData> {
        let user_data = Self::from_id(ctx, user_id).await?;

        Ok(user_data.unwrap_or_else(|| UserData::new(user_id)))
    }

    pub async fn save(&self, ctx: &Context) -> Result<UserData> {
        let pool = ctx.data.read().await.get::<DbPool>().cloned().unwrap();

        upsert_query(&pool, &self).await
    }
}

async fn from_id_query(pool: &sqlx::PgPool, user_id: UserId) -> Result<Option<UserData>> {
    sqlx::query_as!(
        UserData,
        r#"
            SELECT *
              FROM app_public.users
             WHERE id = $1
        "#,
        i64::from(user_id),
    )
    .fetch_optional(pool)
    .await
    .map_err(Into::into)
}

async fn upsert_query(pool: &sqlx::PgPool, user_data: &UserData) -> Result<UserData> {
    sqlx::query_as!(
        UserData,
        r#"
        INSERT INTO app_public.users (id, is_patron, patron_emoji, rep, fishies, last_rep, last_fishies, profile_data, lastfm_username)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        ON CONFLICT (id)
          DO UPDATE
                SET is_patron = $2,
                    patron_emoji = $3,
                    rep = $4,
                    fishies = $5,
                    last_rep = $6,
                    last_fishies = $7,
                    profile_data = $8,
                    lastfm_username = $9
          RETURNING *
        "#,
        user_data.id,
        user_data.is_patron,
        user_data.patron_emoji,
        user_data.rep,
        user_data.fishies,
        user_data.last_rep,
        user_data.last_fishies,
        user_data.profile_data,
        user_data.lastfm_username,
    )
    .fetch_one(pool)
    .await
    .map_err(Into::into)
}
