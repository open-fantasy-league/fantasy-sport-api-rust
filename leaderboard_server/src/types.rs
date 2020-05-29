use crate::schema::*;
use chrono::{DateTime, Utc};
use diesel_utils::{my_timespan_format, my_timespan_format_opt, DieselTimespan};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable)]
#[primary_key(leaderboard_id)]
#[table_name = "leaderboards"]
pub struct Leaderboard {
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(leaderboard_id, player_id, timestamp)]
#[table_name = "stats"]
#[belongs_to(Leaderboard)]
pub struct Stat {
    pub player_id: Uuid,
    pub leaderboard_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub points: f32,
    pub meta: serde_json::Value,
}

#[derive(Deserialize, Debug, AsChangeset)]
#[primary_key(leaderboard_id)]
#[table_name = "leaderboards"]
pub struct LeaderboardUpdate {
    pub leaderboard_id: Uuid,
    pub league_id: Option<Uuid>,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiStat {
    pub player_id: Uuid,
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub points: f32,
    pub meta: serde_json::Value,
}

impl ApiStat {
    pub fn from_rows(rows: Vec<(Leaderboard, Vec<Stat>)>) -> Vec<Self> {
        rows.into_iter()
            .flat_map(|(l, stats)| {
                let league_id = l.league_id;
                stats
                    .into_iter()
                    .map(|s| Self {
                        league_id,
                        player_id: s.player_id,
                        leaderboard_id: s.leaderboard_id,
                        timestamp: s.timestamp,
                        points: s.points,
                        meta: s.meta,
                    })
                    .collect_vec()
            })
            .collect()
    }
}

#[derive(Serialize, Debug)]
pub struct ApiLeaderboard {
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    pub stats: Vec<Stat>,
}

impl ApiLeaderboard {
    pub fn from(rows: (Leaderboard, Vec<Stat>)) -> Self {
        let (leaderboard, stats) = rows;
        Self {
            leaderboard_id: leaderboard.leaderboard_id,
            league_id: leaderboard.league_id,
            name: leaderboard.name,
            meta: leaderboard.meta,
            stats: stats,
        }
    }
}
