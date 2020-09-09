use serenity::async_trait;
use serenity::prelude::*;
use std::sync::Arc;

use crate::error::Result;

#[async_trait]
pub trait SushiiConfigDb {
    async fn get(ctx: &Context) -> Arc<SushiiConfig>;
}

#[derive(Debug, Clone)]
pub struct SushiiConfig {
    pub discord_token: String,
    pub owner_ids: Vec<u64>,
    pub database_url: String,
    pub default_prefix: String,
    pub blocked_users: Vec<u64>,
    pub lastfm_key: String,
    pub metrics_port: u16,
}

fn parse_id_array(s: &str) -> Vec<u64> {
    s.split(',')
        .collect::<Vec<&str>>()
        .iter()
        .filter_map(|u| u.trim().parse::<u64>().ok())
        .collect()
}

impl SushiiConfig {
    pub fn new_from_env() -> Result<Self> {
        if let Err(e) = dotenv::dotenv() {
            tracing::warn!("Failed to read .env file: {}", e);
        }

        Ok(SushiiConfig {
            discord_token: dotenv::var("DISCORD_TOKEN")?,
            owner_ids: parse_id_array(&dotenv::var("OWNER_IDS").unwrap_or_else(|_| "".into())),
            database_url: dotenv::var("DATABASE_URL")?,
            default_prefix: dotenv::var("DEFAULT_PREFIX")?,
            blocked_users: parse_id_array(
                &dotenv::var("BLOCKED_USERS").unwrap_or_else(|_| "".into()),
            ),
            lastfm_key: dotenv::var("LASTFM_KEY").unwrap_or_else(|_| "".into()),
            metrics_port: dotenv::var("METRICS_PORT")
                .ok()
                .and_then(|x| x.parse().ok())
                .unwrap_or(9888),
        })
    }
}

#[async_trait]
impl SushiiConfigDb for SushiiConfig {
    async fn get(ctx: &Context) -> Arc<Self> {
        let data = ctx.data.read().await;

        data.get::<SushiiConfig>()
            .expect("Context data is missing SushiiConfig")
            .clone()
    }
}

#[test]
fn parses_array() {
    let expected = vec![123, 456, 789];
    assert_eq!(parse_id_array("123,456,789"), expected);
    assert_eq!(parse_id_array("123, 456, 789"), expected);
    assert_eq!(parse_id_array("123, 456   , 789         "), expected);
}
