use serde::{Deserialize, Serialize};
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use super::{matches::Match, series::Series};

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations, Clone)]
#[primary_key(match_id, team_id)]
#[belongs_to(Match)]
#[table_name = "team_match_results"]
pub struct TeamMatchResult {
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: Option<String>,
    pub meta: serde_json::Value,
}


#[derive(Deserialize, Debug, Identifiable, Associations, AsChangeset)]
#[primary_key(match_id, team_id)]
#[belongs_to(Match)]
#[table_name = "team_match_results"]
pub struct TeamMatchResultUpdate {
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: Option<String>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations, Clone)]
#[primary_key(series_id, team_id)]
#[belongs_to(Series)]
#[table_name = "team_series_results"]
pub struct TeamSeriesResult {
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: Option<String>,
    pub meta: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Identifiable, Associations, AsChangeset)]
#[primary_key(series_id, team_id)]
#[belongs_to(Series)]
#[table_name = "team_series_results"]
pub struct TeamSeriesResultUpdate {
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: Option<String>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations, Clone)]
#[primary_key(match_id, player_id)]
#[belongs_to(Match)]
#[table_name = "player_results"]
pub struct PlayerResult {
    pub player_id: Uuid,
    pub match_id: Uuid,
    pub result: Option<serde_json::Value>,
    pub meta: serde_json::Value,
}

#[derive(Deserialize, Debug, Identifiable, Associations, AsChangeset)]
#[primary_key(match_id, player_id)]
#[belongs_to(Match)]
#[table_name = "player_results"]
pub struct PlayerResultUpdate {
    pub player_id: Uuid,
    pub match_id: Uuid,
    pub result: Option<serde_json::Value>,
    pub meta: Option<serde_json::Value>,
}