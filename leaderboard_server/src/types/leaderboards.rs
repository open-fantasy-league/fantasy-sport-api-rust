use crate::schema::*;
use serde_json;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use diesel_utils::{DieselTimespan, my_timespan_format, my_timespan_format_opt, PgConn};

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(leaderboard_id)]
pub struct Leaderboard {
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan
}

#[derive(Deserialize, Debug, AsChangeset)]
#[primary_key(leaderboard_id)]
#[table_name = "leaderboards"]
pub struct LeaderboardUpdate {
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(Leaderboard)]
#[primary_key(leaderboard_id, player_id, timestamp)]
pub struct Stat {
    pub player_id: Uuid,
    pub leaderboard_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub points: f32,
    pub meta: serde_json::Value,
}

#[derive(Serialize, Debug, LabelledGeneric)]
pub struct ApiLeaderboard{
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    pub stats: Vec<Stat>
}

impl ApiLeaderboard{

    pub fn from(rows: (Leaderboard, Vec<Stat>)) -> Self{
        let (leaderboard, stats) = rows;
        Self{
            leaderboard_id: leaderboard.leaderboard_id,
            league_id: leaderboard.league_id,
            name: leaderboard.name,
            meta: leaderboard.meta,
            stats: stats
        }
    }
}