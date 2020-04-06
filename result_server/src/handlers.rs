use serde::{Deserialize, Serialize};
use crate::utils::my_timespan_format;
use crate::db;
use chrono::{DateTime, Utc};
use crate::db_pool::PgConn;
use std::collections::Bound;
use frunk::labelled::transform_from;
use crate::models::*;
use uuid::Uuid;

use std::convert::From;

#[derive(Serialize)]
struct ErrorResp {
    code: u16,
    message: String,
}

#[derive(Deserialize, LabelledGeneric)]
pub struct ApiNewCompetition{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Deserialize, LabelledGeneric)]
pub struct ApiNewSeries{
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
    pub teams: Vec<String>,
}

impl From<ApiNewCompetition> for DbNewCompetition{
    fn from(x: ApiNewCompetition) -> Self{
        DbNewCompetition{code: x.code, name: x.name, meta: x.meta, timespan: x.timespan}
    }
}

impl From<ApiNewSeries> for DbNewSeries{
    fn from(x: ApiNewSeries) -> Self{
        DbNewSeries{competition_id: x.competition_id, code: x.code, name: x.name, meta: x.meta, timespan: x.timespan}
    }
}

//impl to other type

pub async fn create_series(series: ApiNewSeries, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    //let teams = db::find_teams(&conn, series.teams);
    let created = db::create_series(&conn, &transform_from(series));
    //let created = db::create_series(&conn, &DbNewSeries::from(series));
    match created{
        Ok(c) => Ok(warp::reply::json(&c)),
        Err(e) => Ok(warp::reply::json(&ErrorResp{code:500, message: e.to_string()}))
    }
}

pub async fn create_competition(comp: ApiNewCompetition, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_competition(&conn, &DbNewCompetition::from(comp));
    match created{
        Ok(c) => Ok(warp::reply::json(&c)),
        Err(e) => Ok(warp::reply::json(&ErrorResp{code:500, message: e.to_string()}))
    }
}


