use crate::schema::*;
use serde_json;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::publisher::Publishable;
use diesel_utils::PgConn;
use warp_ws_server::BoxError;

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(leaderboard_id)]
pub struct Leaderboard {
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
}

#[derive(Deserialize, Debug, AsChangeset)]
#[primary_key(leaderboard_id)]
#[table_name = "leaderboards"]
pub struct LeaderboardUpdate {
    pub leaderboard_id: Uuid,
    pub league_id: Uuid,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
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

// // This is shit and a consequence of not splitting up subscriptions
// #[derive(Serialize, Debug, LabelledGeneric)]
// pub struct ApiLeaderboard2ElectricBoogaloo{
//     pub leaderboard_id: Uuid,
//     pub league_id: Uuid,
//     pub name: String,
//     pub meta: serde_json::Value,
//     pub points: Vec<Points>
// }

impl Publishable for Leaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard"
    }

    fn subscription_map_key(&self) -> Uuid {
        self.league_id
    }

    fn subscription_id_map(
        conn: Option<&PgConn>,
        publishables: &Vec<Self>,
    ) -> Result<HashMap<Uuid, Uuid>, BoxError> {
        Ok(publishables
            .iter()
            .map(|c| (c.league_id, c.league_id))
            .collect())
    }
}

impl Publishable for ApiLeaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard_detailed"
    }

    fn subscription_map_key(&self) -> Uuid {
        self.league_id
    }

    fn subscription_id_map(
        conn: Option<&PgConn>,
        publishables: &Vec<Self>,
    ) -> Result<HashMap<Uuid, Uuid>, BoxError> {
        Ok(publishables
            .iter()
            .map(|c| (c.league_id, c.league_id))
            .collect())
    }
}

impl Publishable for Stat {
    fn message_type<'a>() -> &'a str {
        "stat"
    }

    fn subscription_map_key(&self) -> Uuid {
        self.leaderboard_id
    }

    fn subscription_id_map(
        conn: Option<&PgConn>,
        publishables: &Vec<Self>,
    ) -> Result<HashMap<Uuid, Uuid>, BoxError> {
        Ok(publishables
            .iter()
            .map(|c| (c.leaderboard_id, c.leaderboard_id))
            .collect())
    }
}