use serde::{Deserialize, Serialize};
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use warp_ws_server::utils::my_timespan_format_opt;
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use super::{matches::Match, series::Series};
use crate::publisher::Publishable;

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(match_id, team_id)]
#[belongs_to(Match<'a>)]
#[table_name = "team_match_results"]
pub struct TeamMatchResult<'a> {
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(series_id, team_id)]
#[belongs_to(Series)]
#[table_name = "team_series_results"]
pub struct TeamSeriesResult {
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(match_id, player_id)]
#[belongs_to(Match<'a>)]
#[table_name = "player_results"]
pub struct PlayerResult<'a> {
    pub player_id: Uuid,
    pub match_id: Uuid,
    pub result: serde_json::Value,
    pub meta: serde_json::Value,
}

impl<'a> Publishable for TeamMatchResult<'a> {
    fn message_type<'a>() -> &'a str {
        "team_match_results"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.match_id
    }
}

impl<'a> Publishable<'a> for PlayerResult<'a> {
    fn message_type<'a>() -> &'a str {
        "player_match_results"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.match_id
    }
}

impl<'a> Publishable for TeamSeriesResult {
    fn message_type<'a>() -> &'a str {
        "team_series_results"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.series_id
    }
}