use serde::{Deserialize, Serialize};
use crate::utils::my_timespan_format;
use crate::db;
use crate::db_pool::PgConn;
use std::collections::HashMap;
use crate::DieselTimespan;
use frunk::labelled::transform_from;
use crate::models::*;
use uuid::Uuid;
use itertools::Itertools;

#[derive(Serialize)]
pub struct ErrorResp {
    pub code: u16,
    pub message: String,
}

#[derive(Serialize)]
pub struct WSMsgOut<'a, T: Serialize> {
    pub message_id: Uuid,
    pub mode: &'a str,
    pub message_type: &'a str,
    pub data: T
}

impl<'a, T: Serialize> WSMsgOut<'a, T>{
    pub fn resp(message_id: Uuid, message_type: &'a str, data: T) -> Self{
        return Self{message_id: message_id, message_type: message_type, mode: "resp", data: data}
    }

    pub fn push(message_type: &'a str, data: T) -> Self{
        return Self{message_id: Uuid::new_v4(), message_type: message_type, mode: "push", data: data}
    }
}


#[derive(Deserialize)]
pub struct WSReq<'a> {
    pub message_id: Uuid,
    pub method: &'a str,
    pub data: serde_json::Value
    //pub data: String
}


#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiSubTeams{
    pub toggle: bool,
}
#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiSubCompetitions{
    pub competition_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>
}

#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiNewCompetition{
    pub competition_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
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

#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiNewTeam{
    pub team_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Debug)]
pub struct ApiTeam{
    pub team_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<ApiTeamName>,
}

impl ApiTeam{
    
    pub fn from_rows(rows: Vec<(Team, TeamName)>) -> Vec<Self>{
        // Group rows by team-id using hashmap, build a list of different team names
        // Assume if a team has no names ever, we dont care about it
        let mut acc: HashMap<Uuid, (Team, Vec<TeamName>)> = HashMap::new();
        acc = rows.into_iter().fold(acc, |mut acc, (team, team_name)| {
            match acc.get_mut(&team.team_id) {
                Some(t) => {t.1.push(team_name);},
                None => {acc.insert(team.team_id, (team, vec![team_name]));},
            }
            acc
        });

        acc.into_iter().map(|(team_id, v)|{
            Self{
                team_id: team_id,
                meta: v.0.meta,
                names: v.1.into_iter().map(|tn| ApiTeamName{name: tn.name, timespan: tn.timespan}).collect_vec()
            }
        })
        .collect_vec()
    }
}

#[derive(Serialize, Debug)]
pub struct ApiTeamName{
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiNewPlayer{
    pub player_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Debug)]
pub struct ApiPlayer{
    pub player_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<ApiPlayerName>,
}

impl ApiPlayer{
    
    pub fn from_rows(rows: Vec<(Player, PlayerName)>) -> Vec<Self>{
        // Group rows by team-id using hashmap, build a list of different team names
        // Assume if a team has no names ever, we dont care about it
        let mut acc: HashMap<Uuid, (Player, Vec<PlayerName>)> = HashMap::new();
        acc = rows.into_iter().fold(acc, |mut acc, (player, player_name)| {
            match acc.get_mut(&player.player_id) {
                Some(t) => {t.1.push(player_name);},
                None => {acc.insert(player.player_id, (player, vec![player_name]));},
            }
            acc
        });

        acc.into_iter().map(|(pid, v)|{
            Self{
                player_id: pid,
                meta: v.0.meta,
                names: v.1.into_iter().map(|tn| ApiPlayerName{name: tn.name, timespan: tn.timespan}).collect_vec()
            }
        })
        .collect_vec()
    }
}

#[derive(Serialize, Debug)]
pub struct ApiPlayerName{
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Debug)]
pub struct ApiTeamsAndPlayers{
    pub teams: Vec<ApiTeam>,
    pub players: Vec<ApiPlayer>,
    pub team_players: Vec<TeamPlayer>
}

//using frunk instead
/*impl From<ApiNewCompetition> for NewCompetition{
    fn from(x: ApiNewCompetition) -> Self{
        NewCompetition{code: x.code, name: x.name, meta: x.meta, timespan: x.timespan}
    }
}*/

fn handle_handling_the_handler<T: Serialize>(what_happened: Result<T, diesel::result::Error>) -> Result<impl warp::Reply, warp::Rejection>{
    match what_happened{
        Ok(yay) => Ok(warp::reply::json(&yay)),
        Err(fuuu) => Ok(warp::reply::json(&ErrorResp{code:500, message: fuuu.to_string()}))
    }
}

//pub async fn upsert_serieses(conn: PgConn, mut new: Vec<ApiNewSeries>) -> Result<impl warp::Reply, warp::Rejection>{
pub async fn upsert_serieses(conn: PgConn, mut new: Vec<ApiNewSeries>) -> Result<Vec<Series>, diesel::result::Error>{
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
    conn.build_transaction().run(|| {
        db::upsert_serieses(
            &conn, new.into_iter().map(transform_from).collect_vec()
        ).and_then(|ser|{
            let num_results = ser.len();
            ser.into_iter().map(|s| {
                match db::upsert_series_teams(&conn, &s.series_id, &series_teams[&s.series_id]){
                    Ok(_) => Ok(s), // still want to return series, with series-id
                    Err(fuuu) => Err(fuuu)
                }
            })
            // I dunno how efficient this is, think map will do all the maps, then fold stops first
            // error.
            // Ideally would want to stop `map`ing as soon as hit error
            .fold_results(Vec::with_capacity(num_results), |mut v, r| {v.push(r); v})
        })
    })
}

pub async fn upsert_competitions(conn: PgConn, new: Vec<ApiNewCompetition>) -> Result<Vec<Competition>, diesel::result::Error>{
    db::upsert_competitions(&conn, new.into_iter().map(transform_from).collect_vec())
}


pub async fn upsert_teams(conn: PgConn, new: Vec<ApiNewTeam>) -> Result<Vec<Team>, diesel::result::Error>{
    conn.build_transaction().run(|| db::upsert_teams(&conn, new))
}

pub async fn upsert_players(conn: PgConn, new: Vec<ApiNewPlayer>) -> Result<Vec<Player>, diesel::result::Error>{
    conn.build_transaction().run(|| db::upsert_players(&conn, new))
}

pub async fn upsert_matches(conn: PgConn, new: Vec<NewMatch>) -> Result<Vec<Match>, diesel::result::Error>{
    db::upsert_matches(&conn, new)
}

// pub async fn upsert_team_players(conn: PgConn, new: Vec<NewTeamPlayer>) -> Result<usize, diesel::result::Error>{
//     db::upsert_team_players(&conn, new)
// }
