use serde::{Deserialize, Serialize};
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use frunk::labelled::transform_from;
use super::series::Series;
use super::competitions::Competition;
use super::results::*;
use diesel_utils::{PgConn, DieselTimespan, my_timespan_format, my_timespan_format_opt};
use itertools::{izip, Itertools};
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use diesel::prelude::*;


#[derive(Insertable, Deserialize, LabelledGeneric, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(Series)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct Match {
    pub match_id: Uuid,
    pub name: String,
    pub series_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, LabelledGeneric, Debug, AsChangeset)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct MatchUpdate {
    pub match_id: Uuid,
    pub series_id: Option<Uuid>,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

#[derive(Deserialize, Serialize, Debug, Clone, LabelledGeneric)]
pub struct ApiMatch{
    pub match_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub player_results: Option<Vec<PlayerResult>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_results: Option<Vec<TeamMatchResult>>
}

#[derive(Deserialize, Serialize, Debug, Clone, LabelledGeneric)]
pub struct ApiMatchNew{
    pub match_id: Uuid,
    pub series_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub player_results: Option<Vec<PlayerResult>>,
    pub team_results: Option<Vec<TeamMatchResult>>
}

impl ApiMatch{
    pub fn insertable(self, series_id: Uuid) -> (Match, Vec<PlayerResult>, Vec<TeamMatchResult>){
        (
            Match{match_id: self.match_id, name: self.name, meta: self.meta, timespan: self.timespan, series_id},
            self.player_results.unwrap_or(vec![]),
            self.team_results.unwrap_or(vec![])
        )
    }
}

impl ApiMatchNew{
    pub fn insert(conn: &PgConn, new: Vec<ApiMatchNew>) -> Result<Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>, diesel::result::Error>{
        let (mut player_results, mut team_match_results) = (vec![], vec![]);
        let matches: Vec<Match> = new
            .into_iter().map(|m|{
                let series_id = m.series_id;
                let m2: ApiMatch = transform_from(m);
                let mut tup = m2.insertable(series_id);
                player_results.append(&mut tup.1);
                team_match_results.append(&mut tup.2);
                tup.0
            }).collect_vec();
        let _: Vec<Match> = insert!(conn, matches::table, &matches)?;
        let _: Vec<PlayerResult> = insert!(conn, player_results::table, &player_results)?;
        let _: Vec<TeamMatchResult> = insert!(conn, team_match_results::table, &team_match_results)?;
        let grouped_presults = player_results.grouped_by(&matches);
        let grouped_tresults = team_match_results.grouped_by(&matches);
        let out = izip!(matches, grouped_presults, grouped_tresults).collect();
        Ok(out)
    }
}

pub type CompetitionHierarchyMatchRow = (
    Competition,
    Vec<(
        Series,
        Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>,
    )>,
);

#[derive(Deserialize, Serialize, Debug, Clone, LabelledGeneric)]
pub struct ApiMatchHierarchy{
    pub competition_id: Uuid,
    pub series_id: Uuid,
    pub match_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub player_results: Vec<PlayerResult>,
    pub team_results: Vec<TeamMatchResult>
}
