use crate::types::*;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct SubLeague {
    pub sub_league_ids: Option<Vec<Uuid>>,
    pub unsub_league_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct SubLeaderboard {
    pub sub_leaderboard_ids: Option<Vec<Uuid>>,
    pub unsub_leaderboard_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>,
}

#[derive(Deserialize)]
#[serde(tag = "method")]
pub enum WSReq {
    Leaderboard {
        message_id: Uuid,
        data: Vec<Leaderboard>,
    },
    LeaderboardUpdate {
        message_id: Uuid,
        data: Vec<LeaderboardUpdate>,
    },
    LeaderboardGet {
        message_id: Uuid,
        data: Vec<Uuid>,
    },
    Stat {
        message_id: Uuid,
        data: Vec<Stat>,
    },
    SubLeague {
        message_id: Uuid,
        data: SubLeague,
    },
    SubLeaderboard {
        message_id: Uuid,
        data: SubLeaderboard,
    },
}
