use serde::{Deserialize, Serialize};
use crate::db;
use std::collections::HashMap;
use warp_ws_server::*;
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use frunk::labelled::transform_from;
use crate::WSConnections_;
use uuid::Uuid;
use itertools::Itertools;
use crate::subscriptions::*;
use crate::publisher::*;
use crate::schema;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use crate::types::competitions::ApiCompetition;

#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiSubTeams{
    pub toggle: bool,
}
#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiSubCompetitions{
    pub competition_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>
}

// #[derive(Deserialize, LabelledGeneric, Debug)]
// pub struct ApiNewPlayer{
//     pub player_id: Option<Uuid>,
//     pub name: String,
//     pub meta: serde_json::Value,
//     #[serde(with = "my_timespan_format")]
//     // Its naive having initial player position and their name share a timespan,
//     // but fudge it! Improve later
//     pub timespan: DieselTimespan,
//     pub position: Option<String>
// }

// impl ApiPlayer{
    
//     pub fn from_rows(rows: Vec<(Player, PlayerName, PlayerPosition)>) -> Vec<Self>{
//         // Group rows by team-id using hashmap, build a list of different team names
//         // Assume if a team has no names ever, we dont care about it
//         let mut acc: HashMap<Uuid, (Player, Vec<PlayerName>, Vec<PlayerPosition>)> = HashMap::new();
//         acc = rows.into_iter().fold(acc, |mut acc, (player, player_name, position)| {
//             match acc.get_mut(&player.player_id) {
//                 Some(t) => {t.1.push(player_name); t.2.push(position)},
//                 None => {acc.insert(player.player_id, (player, vec![player_name], vec![position]));},
//             }
//             acc
//         });

//         acc.into_iter().map(|(pid, v)|{
//             Self{
//                 player_id: pid,
//                 meta: v.0.meta,
//                 names: v.1.into_iter().map(|tn| ApiPlayerName{name: tn.name, timespan: tn.timespan}).collect_vec()
//             }
//         })
//         .collect_vec()
//     }
// }

// pub async fn insert_competitions_with_children(conn: PgConn, new: Vec<ApiCompetition>) -> Result<bool, diesel::result::Error>{
//     let insert_res = ApiCompetition::insert(conn, new).await;
//     // if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
//     //     sub_to_competitions(ws_user, new.iter().map(|c| &c.competition_id)).await;
//     // }
//     // // TODO ideally would return response before awaiting publishing going out
//     publish_competitions(ws_conns, &new).await;
//     // println!("{:?}", &new);
//     // let resp_msg = WSMsgOut::resp(req.message_id, req.method, new);
//     // serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

//pub async fn upsert_serieses(conn: PgConn, mut new: Vec<ApiNewSeries>) -> Result<impl warp::Reply, warp::Rejection>{
// pub async fn upsert_series_with_children(conn: PgConn, mut new: Vec<ApiNewSeries>) -> Result<Vec<Series>, diesel::result::Error>{
//     // This just returns list of raw-series created (without the info on teams for each series)
//     // Due to simplicity meaning either teams-in-series either match the input, or an error
//     // happened

//     //team_ids 7337c529-2972-422f-94a0-247f3a58d001, 7337c529-2972-422f-94a0-247f3a58d002
//     // Not leaving uuid gen to postgresql, so that can tie the teams to individual series created.
//     // However for simple cases like this, returning order should match insertion order
//     // https://dba.stackexchange.com/questions/95822/does-postgres-preserve-insertion-order-of-records
//     // Therefore TODO just enumerate returning, indexing new to find teams
//     // Cloning and pulling out here is necessary, 
//     // because the frunk `transform_from` consumes the old struct
//     // unwrap safe due to above uuidv4 generation
//     let series_teams: HashMap<Uuid, Vec<Uuid>> = new.iter().map(|s| (s.series_id.unwrap(), s.teams.clone())).collect();
//     conn.build_transaction().run(|| {
//         db::upsert_serieses(
//             &conn, new.into_iter().map(transform_from).collect_vec()
//         ).and_then(|ser|{
//             let num_results = ser.len();
//             ser.into_iter().map(|s| {
//                 match db::upsert_series_teams(&conn, &s.series_id, &series_teams[&s.series_id]){
//                     Ok(_) => Ok(s), // still want to return series, with series-id
//                     Err(fuuu) => Err(fuuu)
//                 }
//             })
//             // I dunno how efficient this is, think map will do all the maps, then fold stops first
//             // error.
//             // Ideally would want to stop `map`ing as soon as hit error
//             .fold_results(Vec::with_capacity(num_results), |mut v, r| {v.push(r); v})
//         })
//     })
// }

// pub async fn sub_competitions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: ApiSubCompetitions = serde_json::from_value(req.data)?;
//     // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
//     // why does this need splitting into two lines?
//     // ANd is it holding the lock for this whole scope? doesnt need to
//     let mut hmmmm = ws_conns.lock().await;
//     let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
//     if let Some(toggle) = deserialized.all{
//         sub_to_all_competitions(ws_user, toggle).await;
//     }
//     else if let Some(competition_ids) = deserialized.competition_ids{
//         sub_to_competitions(ws_user, competition_ids.iter()).await;
//     }
//     else{
//         return Err(Box::new(InvalidRequestError{description: String::from("sub_competitions must specify either 'all' or 'competition_ids'")}))
//     }
//     let all_competitions = db::get_all_competitions(&conn)?;
//     let subscribed_comps: Vec<&Competition> = subscribed_comps(&ws_user.subscriptions, &all_competitions);
//     let comp_rows = db::get_full_competitions(
//         &conn,
//             subscribed_comps.iter().map(|x| x.competition_id).collect()
//     )?;
//     let data = ApiCompetition::from_rows(comp_rows);
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn sub_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: ApiSubTeams = serde_json::from_value(req.data)?;
//     let mut hmmmm = ws_conns.lock().await;
//     let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
//     println!("{:?}", &deserialized);
//     sub_to_teams(ws_user, deserialized.toggle).await;

//     let team_out = db::get_all_teams(&conn).map(|rows| ApiTeam::from_rows(rows))?;
//     let players_out = db::get_all_players(&conn).map(|rows| ApiPlayer::from_rows(rows))?;
//     let team_players_out = db::get_all_team_players(&conn)?;
//     let data = ApiTeamsAndPlayers{teams: team_out, players: players_out, team_players: team_players_out};
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

pub async fn insert_competitions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let comps: Vec<ApiCompetition> = serde_json::from_value(req.data)?;
    println!("{:?}", &comps);
    let insert_res = ApiCompetition::insert(conn, &comps).await?;
   // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, deserialized)?;
    // assume anything upserted the user wants to subscribe to
    if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
        sub_to_competitions(ws_user, comps.iter().map(|c| &c.competition_id)).await;
    }
    // TODO ideally would return response before awaiting publishing going out
    publish_competitions(ws_conns, &comps).await;
    println!("{:?}", &comps);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, comps);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// pub async fn update_competitions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: Vec<UpdateCompetition> = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let comps: Vec<Competition> = conn.build_transaction().run(|| {
//         deserialized.iter().map(|c| {
//         update!(&conn, competitions, competition_id, c)
//     }).collect::<Result<Vec<Competition>, _>>()})?;
//     //let comps = db::insert_competitions(&conn, deserialized.into_iter().map(transform_from).collect_vec())?;
//     // assume anything upserted the user wants to subscribe to
//     if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
//         sub_to_competitions(ws_user, comps.iter().map(|c| &c.competition_id)).await;
//     }
//     // TODO ideally would return response before awaiting publishing going out
//     publish_competitions(ws_conns, &comps).await;
//     println!("{:?}", &comps);
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, comps);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn insert_series(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let series_out= upsert_series_with_children(conn, deserialized).await?;
//     //let comp_ids: HashSet<Uuid> = series_out.iter().map(|s| s.competition_id).dedup().collect();
//     // assume anything upserted the user wants to subscribe to
//     // TODO check how turn map into iter
//     if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
//         sub_to_competitions(ws_user, series_out.iter().map(|s| &s.competition_id)).await;
//     }
//     publish_series(ws_conns, &series_out).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, series_out);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }
// pub async fn upsert_matches(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     // TODO async db funkys  upsert_matches(&conn, d).await;
//     let upserted= db::upsert_matches(&conn, deserialized)?;
//     let series_ids: Vec<Uuid> = upserted.iter().map(|s| s.series_id).dedup().collect();
//     let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
//     // assume anything upserted the user wants to subscribe to
//     if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
//         sub_to_competitions(ws_user, comp_and_series_ids.iter().map(|x| &x.1)).await;
//     }
//     publish_matches(ws_conns, &upserted, comp_and_series_ids.into_iter().collect()).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let upserted= db::upsert_teams(&conn, deserialized)?;
//     publish_teams(ws_conns, &upserted).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn insert_players(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: Vec<ApiPlayerIn> = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let upserted= db::insert_players(&conn, deserialized)?;
//     publish_players(ws_conns, &upserted).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_players(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let upserted= db::upsert_players(&conn, deserialized)?;
//     publish_players(ws_conns, &upserted).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_team_players(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let upserted= db::upsert_team_players(&conn, deserialized)?;
//     publish_team_players(ws_conns, &upserted).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_team_match_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: Vec<NewTeamMatchResult> = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let match_ids: Vec<Uuid> = deserialized.iter().map(|x| x.match_id).collect();
//     let upserted= db::upsert_team_match_results(&conn, deserialized)?;
//     let competition_n_match_ids = db::get_competition_ids_for_matches(&conn, &match_ids)?;
//     let comp_to_match_ids: HashMap<Uuid, Uuid> = competition_n_match_ids.into_iter().collect();
//     publish_results::<TeamMatchResult>(ws_conns, &upserted, comp_to_match_ids).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_player_match_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: Vec<NewPlayerResult> = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let match_ids: Vec<Uuid> = deserialized.iter().map(|x| x.match_id).collect();
//     let upserted= db::upsert_player_match_results(&conn, deserialized)?;
//     let competition_n_match_ids = db::get_competition_ids_for_matches(&conn, &match_ids)?;
//     let comp_to_match_ids: HashMap<Uuid, Uuid> = competition_n_match_ids.into_iter().collect();
//     publish_results::<PlayerResult>(ws_conns, &upserted, comp_to_match_ids).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_team_series_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: Vec<NewTeamSeriesResult> = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     let series_ids: Vec<Uuid> = deserialized.iter().map(|x| x.series_id).collect();
//     let upserted= db::upsert_team_series_results(&conn, deserialized)?;
//     let competition_n_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
//     let comp_to_series_ids: HashMap<Uuid, Uuid> = competition_n_series_ids.into_iter().collect();
//     publish_results::<TeamSeriesResult>(ws_conns, &upserted, comp_to_series_ids).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, upserted);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }

// pub async fn upsert_series_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     Ok("dog".to_string())
// }