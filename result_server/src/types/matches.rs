use serde::{Deserialize, Serialize};
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use warp_ws_server::utils::my_timespan_format_opt;
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use super::series::Series;
use super::results::{TeamMatchResult, PlayerResult};
use crate::publisher::Publishable;


#[derive(Insertable, Deserialize, LabelledGeneric, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(Series)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct Match<'a> {
    pub match_id: &'a Uuid,
    pub name: &'a String,
    pub series_id: &'a Uuid,
    pub meta: &'a serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: &'a DieselTimespan,
}

#[derive(Deserialize, LabelledGeneric, Debug, AsChangeset)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct UpdateMatch {
    pub match_id: Uuid,
    pub series_id: Option<Uuid>,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiMatch{
    pub match_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub player_results: Vec<PlayerResult>,
    pub team_results: Vec<TeamMatchResult>
}

impl ApiMatch{
    pub fn insertable(&self, series_id: Uuid) -> (Match, Vec<PlayerResult>, Vec<TeamMatchResult>){
        (
            Match{match_id: self.match_id, name: &self.name, meta: self.meta, timespan: self.timespan, series_id},
            self.player_results,
            self.team_results
        )
    }
}

impl Publishable for ApiMatch {
    fn message_type<'a>() -> &'a str {
        "matches"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.match_id
    }
}