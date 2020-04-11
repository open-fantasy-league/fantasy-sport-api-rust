use serde::{Deserialize, Serialize};
use crate::utils::my_timespan_format;
use crate::db;
use chrono::{DateTime, Utc};
use crate::db_pool::PgConn;
use std::collections::{Bound, HashMap};
use frunk::labelled::transform_from;
use crate::models::*;
use uuid::Uuid;
use itertools::Itertools;

#[derive(Serialize)]
pub struct ErrorResp {
    pub code: u16,
    pub message: String,
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
    pub teams: Vec<Uuid>,
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
}*/

fn handle_handling_the_handler<T: Serialize>(what_happened: Result<T, diesel::result::Error>) -> Result<impl warp::Reply, warp::Rejection>{
    match what_happened{
        Ok(yay) => Ok(warp::reply::json(&yay)),
        Err(fuuu) => Ok(warp::reply::json(&ErrorResp{code:500, message: fuuu.to_string()}))
    }
}

pub async fn create_serieses(mut new: Vec<ApiNewSeries>, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    // This just returns list of raw-series created (without the info on teams for each series)
    // Due to simplicity meaning either teams-in-series either match the input, or an error
    // happened

    //team_ids 7337c529-2972-422f-94a0-247f3a58d001, 7337c529-2972-422f-94a0-247f3a58d002
    // Not leaving uuid gen to postgresql, so that can tie the teams to individual series created.
    // However for simple cases like this, returning order should match insertion order
    // https://dba.stackexchange.com/questions/95822/does-postgres-preserve-insertion-order-of-records
    // Therefore TODO just enumerate returning, indexing new to find teams
    new.iter_mut().for_each(|s| {s.series_id = s.series_id.or(Some(Uuid::new_v4()))});
    // Cloning and pulling out here is necessary, 
    // because the frunk `transform_from` consumes the old struct
    // unwrap safe due to above uuidv4 generation
    let series_teams: HashMap<Uuid, Vec<Uuid>> = new.iter().map(|s| (s.series_id.unwrap(), s.teams.clone())).collect();
    let created = conn.build_transaction().run(|| {db::create_serieses(&conn, new.into_iter().map(transform_from).collect_vec()).and_then(|ser|{
        let num_results = ser.len();
        ser.into_iter().map(|s| {
            match db::create_series_teams(&conn, &s.series_id, &series_teams[&s.series_id]){
                Ok(_) => Ok(s), // still want to return series, with series-id
                Err(fuuu) => Err(fuuu)
            }
        })
        // I dunno how efficient this is, think map will do all the maps, then fold stops first
        // error.
        // Ideally would want to stop `map`ing as soon as hit error
        .fold_results(Vec::with_capacity(num_results), |mut v, r| {v.push(r); v})
    })
    });
    handle_handling_the_handler::<Vec<DbSeries>>(created)
}

pub async fn create_competitions(new: Vec<ApiNewCompetition>, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_competitions(&conn, new.into_iter().map(transform_from).collect_vec());
    handle_handling_the_handler::<Vec<DbCompetition>>(created)
}


pub async fn create_teams(new: Vec<ApiNewTeam>, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_teams(&conn, new.into_iter().map(transform_from).collect_vec());
    handle_handling_the_handler::<Vec<DbTeam>>(created)
}

pub async fn create_players(new: Vec<DbNewPlayer>, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_players(&conn, new);
    handle_handling_the_handler::<Vec<DbPlayer>>(created)
}

pub async fn create_matches(new: Vec<DbNewMatch>, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_matches(&conn, new);
    handle_handling_the_handler::<Vec<DbMatch>>(created)
}

pub async fn create_team_players(new: Vec<DbNewTeamPlayer>, conn: PgConn) -> Result<impl warp::Reply, warp::Rejection>{
    let created = db::create_team_players(&conn, new);
    handle_handling_the_handler::<usize>(created)
}
