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
use crate::types::{competitions::*, series::*, teams::*, matches::*, results::*, players::*};

#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiSubTeams{
    pub toggle: bool,
}
#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiSubCompetitions{
    pub competition_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>
}

// Size for Self cannot be known at compile time.... :L
// #[async_trait]
// pub trait ServerInsertable{
//     async fn insert(conn: &PgConn, new: Vec<Self>) -> Result<bool, diesel::result::Error>;
//     fn comp_id_map_tup(
//         conn: PgConn,
//         me: &Vec<Self>,
//     ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error>;

// }

pub async fn insert_competitions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let comps: Vec<ApiCompetition> = serde_json::from_value(req.data)?;
    println!("{:?}", &comps);
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiCompetition::insert(conn, comps.clone()).await?;
    // TODO ideally would return response before awaiting publishing going out
    publish_competitions(ws_conns, &comps).await;
    println!("{:?}", &comps);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, comps);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_competitions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<UpdateCompetition> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let comps: Vec<Competition> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, competitions, competition_id, c)
    }).collect::<Result<Vec<Competition>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    publish_for_comp::<Competition>(
        ws_conns, &comps,
         comps.iter().map(|c| (c.competition_id, c.competition_id)).collect()
        ).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, comps);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_series(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiSeriesNew> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiSeriesNew::insert(&conn, deserialized.clone())?;
   // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, deserialized)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    let comp_and_series_ids = db::get_competition_ids_for_series(
        &conn, &deserialized.iter().map(|s|s.series_id).dedup().collect()
    )?;
    publish_for_comp::<ApiSeriesNew>(ws_conns, &deserialized, comp_and_series_ids.into_iter().collect()).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_series(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<SeriesUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<Series> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, series, series_id, c)
    }).collect::<Result<Vec<Series>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    publish_for_comp::<Series>(
        ws_conns, &out,
        out.iter().map(|c| (c.series_id, c.competition_id)).collect()
        ).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_matches(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiMatchNew> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiMatchNew::insert(&conn, deserialized.clone())?;
   // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, deserialized)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    let series_ids: Vec<Uuid> = deserialized.iter().map(|s| s.series_id).dedup().collect();
    let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    publish_for_comp::<ApiMatchNew>(ws_conns, &deserialized, comp_and_series_ids.into_iter().collect()).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_matches(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<UpdateMatch> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<Match> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, matches, match_id, c)
    }).collect::<Result<Vec<Match>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    let series_ids: Vec<Uuid> = out.iter().map(|s| s.series_id).dedup().collect();
    let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    publish_for_comp::<Match>(
        ws_conns, &out,
        comp_and_series_ids.into_iter().collect()
        ).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_series_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<TeamSeriesResult> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let series_ids: Vec<Uuid> = deserialized.iter().map(|x| x.series_id).collect();
    insert_exec!(&conn, schema::team_series_results::table, &deserialized)?;
    let competition_n_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    let comp_to_series_ids: HashMap<Uuid, Uuid> = competition_n_series_ids.into_iter().collect();
    publish_for_comp::<TeamSeriesResult>(ws_conns, &deserialized, comp_to_series_ids).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_series_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<TeamSeriesResultUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let series_ids: Vec<Uuid> = deserialized.iter().map(|x| x.series_id).collect();
    let out: Vec<TeamSeriesResult> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
            update_2pkey!(&conn, team_series_results, series_id, team_id, c)
    }).collect::<Result<Vec<TeamSeriesResult>, _>>()})?;
    let competition_n_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    let comp_to_series_ids: HashMap<Uuid, Uuid> = competition_n_series_ids.into_iter().collect();
    publish_for_comp::<TeamSeriesResult>(ws_conns, &out, comp_to_series_ids).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_match_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<TeamMatchResult> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let match_ids: Vec<Uuid> = deserialized.iter().map(|x| x.match_id).collect();
    insert_exec!(&conn, schema::team_match_results::table, &deserialized)?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish_for_comp::<TeamMatchResult>(ws_conns, &deserialized, comp_id_map).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_match_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<TeamMatchResultUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let match_ids: Vec<Uuid> = deserialized.iter().map(|x| x.match_id).collect();
    let out: Vec<TeamMatchResult> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
            update_2pkey!(&conn, team_match_results, match_id, team_id, c)
    }).collect::<Result<Vec<TeamMatchResult>, _>>()})?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish_for_comp::<TeamMatchResult>(ws_conns, &out, comp_id_map).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<PlayerResult> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let match_ids: Vec<Uuid> = deserialized.iter().map(|x| x.match_id).collect();
    insert_exec!(&conn, schema::player_results::table, &deserialized)?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish_for_comp::<PlayerResult>(ws_conns, &deserialized, comp_id_map).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_player_results(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<PlayerResultUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let match_ids: Vec<Uuid> = deserialized.iter().map(|x| x.match_id).collect();
    let out: Vec<PlayerResult> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
            update_2pkey!(&conn, player_results, match_id, player_id, c)
    }).collect::<Result<Vec<PlayerResult>, _>>()})?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish_for_comp::<PlayerResult>(ws_conns, &out, comp_id_map).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiTeam> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    ApiTeam::insert(conn, deserialized.clone())?;
    publish_for_teams::<ApiTeam>(ws_conns, &deserialized).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<TeamUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<Team> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
            update!(&conn, teams, team_id, c)
    }).collect::<Result<Vec<Team>, _>>()})?;
    publish_for_teams::<Team>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_players(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiPlayer> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    ApiPlayer::insert(conn, deserialized.clone())?;
    publish_for_teams::<ApiPlayer>(ws_conns, &deserialized).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_players(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<PlayerUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<Player> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
            update!(&conn, players, player_id, c)
    }).collect::<Result<Vec<Player>, _>>()})?;
    publish_for_teams::<Player>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_players(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiTeamPlayer> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out = db::insert_team_players(conn, deserialized).await?;
    publish_for_teams::<TeamPlayer>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_names(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiTeamNameNew> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out = db::insert_team_names(conn, deserialized).await?;
    publish_for_teams::<TeamName>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_names(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiPlayerNameNew> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out = db::insert_player_names(conn, deserialized).await?;
    publish_for_teams::<PlayerName>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_positions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ApiPlayerPositionNew> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out = db::insert_player_positions(conn, deserialized).await?;
    publish_for_teams::<PlayerPosition>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}
// TODO Prob need some deletions

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

// pub async fn upsert_series_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     Ok("dog".to_string())
// }

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

pub async fn sub_competitions(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let deserialized: ApiSubCompetitions = serde_json::from_value(req.data)?;
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = deserialized.all{
        sub_to_all_competitions(ws_user, toggle).await;
    }
    else if let Some(competition_ids) = deserialized.competition_ids{
        sub_to_competitions(ws_user, competition_ids.iter()).await;
    }
    else{
        return Err(Box::new(InvalidRequestError{description: String::from("sub_competitions must specify either 'all' or 'competition_ids'")}))
    }
    let all_competitions = db::get_all_competitions(&conn)?;
    let subscribed_comps: Vec<&Competition> = subscribed_comps::<Competition>(&ws_user.subscriptions, &all_competitions);
    let comp_rows = db::get_full_competitions(
        &conn, subscribed_comps.iter().map(|x| x.competition_id).collect()
    )?;
    let data = ApiCompetition::from_rows(comp_rows);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let deserialized: ApiSubTeams = serde_json::from_value(req.data)?;
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    println!("{:?}", &deserialized);
    sub_to_teams(ws_user, deserialized.toggle).await;

    let team_out = db::get_all_teams(&conn).map(|rows| ApiTeam::from_rows(rows))?;
    let players_out = db::get_all_players(&conn).map(|rows| ApiPlayer::from_rows(rows))?;
    let team_players_out = db::get_all_team_players(&conn)?;
    let data = ApiTeamsAndPlayers{teams: team_out, players: players_out, team_players: team_players_out};
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// Nice idea but Deserilize complains about different liftimes
// TODO Work out why and how to fix
// pub async fn insert_generic<'a, T: ServerInsertable + std::fmt::Debug + Deserialize<'a> + Clone + Publishable + Serialize>(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     let deserialized: Vec<T> = serde_json::from_value(req.data)?;
//     println!("{:?}", &deserialized);
//     // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
//     // It's possible to just borrow it in db-insertion,
//     // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
//     T::insert(&conn, deserialized.clone()).await?;
//    // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, deserialized)?;
//     // assume anything upserted the user wants to subscribe to
//     // TODO ideally would return response before awaiting publishing going out
//     let comp_id_map_tup = T::comp_id_map_tup(
//         conn, &deserialized
//     )?;
//     publish_for_comp::<T>(ws_conns, &deserialized, comp_id_map_tup.into_iter().collect()).await;
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, deserialized);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }