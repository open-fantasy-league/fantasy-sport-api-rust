use serde::{Deserialize, Serialize};
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use warp_ws_server::utils::my_timespan_format_opt;
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use super::{competitions::*, matches::*, results::*, teams::*};
use warp_ws_server::PgConn;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use frunk::labelled::transform_from;
use crate::publisher::Publishable;
use itertools::Itertools;


#[derive(Queryable, Serialize, Deserialize, Insertable, Debug, Identifiable, Associations, LabelledGeneric)]
#[belongs_to(Competition)]
#[primary_key(series_id)]
#[table_name = "series"]
pub struct Series {
    pub series_id: Uuid,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}


#[derive(Deserialize, LabelledGeneric, Debug, AsChangeset)]
#[primary_key(series_id)]
#[table_name = "series"]
pub struct UpdateSeries {
    pub series_id: Uuid,
    pub competition_id: Option<Uuid>,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiSeries{
    pub series_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub matches: Vec<ApiMatch>,
    pub teams: Vec<SeriesTeam>,
    pub team_results: Vec<TeamSeriesResult>,
}

impl ApiSeries{
    pub fn insertable(&self, competition_id: Uuid) -> (Series, Vec<Match>, Vec<PlayerResult>, Vec<TeamMatchResult>, Vec<SeriesTeam>, Vec<TeamSeriesResult>){
        let (mut player_results, mut team_match_results) = (vec![], vec![]);
        let series_id = self.series_id;
        let matches = self.matches
            .into_iter().map(|m| {
                let (new_m, mut new_pr, mut new_tr) = m.insertable(series_id);
                team_match_results.append(&mut new_tr);
                player_results.append(&mut new_pr);
                new_m
            }).collect_vec();
        (
            Series{series_id: self.series_id, name: self.name, meta: self.meta, timespan: self.timespan, competition_id},
            matches, player_results, team_match_results, self.teams, self.team_results
        )
    }
}

#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiNewSeries{
    pub series_id: Option<Uuid>,
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub teams: Vec<Uuid>,
}


#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(series_id, team_id)]
#[belongs_to(Series)]
#[table_name = "series_teams"]
pub struct SeriesTeam {
    series_id: Uuid,
    pub team_id: Uuid,
}

impl Publishable for ApiSeries {
    fn message_type<'a>() -> &'a str {
        "series"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.series_id
    }
}