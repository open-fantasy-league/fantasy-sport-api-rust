use serde::{Deserialize, Serialize};
use crate::utils::my_timespan_format;
use crate::db;
use chrono::{DateTime, Utc};
use crate::db_pool::PgConn;
use std::collections::Bound;
use frunk::labelled::transform_from;
use crate::models::*;
use uuid::Uuid;

#[derive(Serialize)]
struct ErrorResp {
    code: u16,
    message: String,
}

#[derive(Deserialize, LabelledGeneric)]
pub struct ApiNewCompetition{
    pub competition_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Deserialize, LabelledGeneric)]
pub struct ApiNewSeries{
    pub series_id: Option<Uuid>,
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
    pub teams: Vec<String>,
}

#[derive(Deserialize, LabelledGeneric)]
pub struct ApiNewTeam{
    pub team_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

//using frunk instead
/*impl From<ApiNewCompetition> for DbNewCompetition{
    fn from(x: ApiNewCompetition) -> Self{
        DbNewCompetition{code: x.code, name: x.name, meta: x.meta, timespan: x.timespan}
    }
}

impl From<ApiNewSeries> for DbNewSeries{
    fn from(x: ApiNewSeries) -> Self{
        DbNewSeries{competition_id: x.competition_id, code: x.code, name: x.name, meta: x.meta, timespan: x.timespan}
    }
}*/

//impl to other type

fn handle_handling_the_handler<T: Serialize>(what_happened: Result<T, diesel::result::Error>) -> Result<impl warp::Reply, warp::Rejection>{
    match what_happened{
        Ok(yay) => Ok(warp::reply::json(&yay)),
        Err(fuuu) => Ok(warp::reply::json(&ErrorResp{code:500, message: fuuu.to_string()}))
    }
}

pub async fn create_series(series: ApiNewSeries, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    //let teams = db::find_teams(&conn, series.teams);
    //team_ids 7337c529-2972-422f-94a0-247f3a58d001, 7337c529-2972-422f-94a0-247f3a58d002
    let created = db::create_series(&conn, &transform_from(series));
    handle_handling_the_handler::<DbSeries>(created)
}

pub async fn create_competition(comp: ApiNewCompetition, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_competition(&conn, &transform_from(comp));
    handle_handling_the_handler::<DbCompetition>(created)
}


pub async fn create_team(team: ApiNewTeam, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_team(&conn, &transform_from(team));
    handle_handling_the_handler::<DbTeam>(created)
}

