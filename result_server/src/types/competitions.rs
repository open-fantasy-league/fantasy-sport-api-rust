use serde::{Deserialize, Serialize};
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use warp_ws_server::utils::my_timespan_format_opt;
use warp_ws_server::PgConn;
use crate::schema::{self, *};
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use frunk::labelled::transform_from;
use super::{series::*, matches::*, results::*};
use itertools::Itertools;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;

#[derive(Deserialize, Serialize, Debug, LabelledGeneric, Clone)]
pub struct ApiCompetition{
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub series: Vec<ApiSeries>
}

#[derive(Queryable, Serialize, Debug, Identifiable, Associations, Insertable, LabelledGeneric)]
#[primary_key(competition_id)]
#[table_name = "competitions"]
pub struct Competition {
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, LabelledGeneric, Debug, AsChangeset)]
#[primary_key(competition_id)]
#[table_name = "competitions"]
pub struct UpdateCompetition {
    pub competition_id: Uuid,
    pub name: Option<String>,
    pub meta: Option<serde_json::Value>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

pub type CompetitionHierarchy = Vec<(
    Competition,
    Vec<(
        Series,
        Vec<TeamSeriesResult>,
        Vec<SeriesTeam>,
        Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>,
    )>,
)>;

impl ApiCompetition{
    // Vec<(Competition, Vec<(Series, Vec<TeamSeriesResult>, Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>)>)>
    pub fn from_rows(rows: CompetitionHierarchy) -> Vec<Self>{
        rows.into_iter().map(|(c, v)| {
            Self{
                competition_id: c.competition_id, name: c.name, meta: c.meta, timespan: c.timespan,
                series: v.into_iter().map(|(s, tr, st, v)|{
                    ApiSeries{
                        series_id: s.series_id, name: s.name, meta: s.meta, timespan: s.timespan,
                        teams: st, team_results: tr, matches: v.into_iter().map(|(m, pr, tr)|{
                            ApiMatch{
                                match_id: m.match_id, name: m.name, meta: m.meta, timespan: m.timespan,
                                player_results: pr, team_results: tr
                            }
                        }).collect_vec()
                    }
                }).collect_vec()
            }
        }).collect_vec()
    }

    pub async fn insert(conn: PgConn, comps: Vec<ApiCompetition>) -> Result<bool, diesel::result::Error>{
        // Couldnt get awkward flat_map and unzip_n to work properly
        let (
            mut series, mut matches, mut player_results, mut team_match_results,
            mut series_teams, mut team_results
        ) = (vec![], vec![], vec![], vec![], vec![], vec![]);
        let raw_comps: Vec<Competition> = comps.into_iter().map(|c|{
            let competition_id = c.competition_id;
            let mut new_series = c.series.into_iter().map(|s| {
                let (
                    s2, mut new_matches, mut new_player_res, mut new_team_match_res,
                    mut new_teams, mut new_team_results
                ) = s.insertable(competition_id);
                matches.append(&mut new_matches);
                series_teams.append(&mut new_teams);
                team_results.append(&mut new_team_results);
                player_results.append(&mut new_player_res);
                team_match_results.append(&mut new_team_match_res);
                s2
            }).collect_vec();
            series.append(&mut new_series);
            Competition{competition_id, meta: c.meta, name: c.name, timespan: c.timespan}
        }).collect_vec();
        //let raw_comps: Vec<Competition> = comps.into_iter().map(transform_from).collect_vec();
        insert_exec!(&conn, schema::competitions::table, raw_comps)?;
        insert_exec!(&conn, schema::series::table, series)?;
        insert_exec!(&conn, schema::series_teams::table, series_teams)?;
        insert_exec!(&conn, schema::matches::table, matches)?;
        insert_exec!(&conn, schema::player_results::table, player_results)?;
        insert_exec!(&conn, schema::team_match_results::table, team_match_results)?;
        insert_exec!(&conn, schema::team_series_results::table, team_results)?;
        return Ok(true)

    }
}