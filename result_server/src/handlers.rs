use serde::{Deserialize, Serialize};
use crate::db;
use std::collections::HashMap;
use warp_ws_server::*;
use diesel_utils::{PgConn, DieselTimespan, my_timespan_format};
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
use serde_json::json;
use crate::messages::*;


pub async fn insert_competitions(method: &str, message_id: Uuid, data: Vec<ApiCompetition>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiCompetition::insert(conn, data.clone()).await?;
    // TODO ideally would return response before awaiting publishing going out
    publish::<SubType, ApiCompetition>(
        ws_conns, &data, SubType::Competition, None
    ).await;
    println!("{:?}", &data);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_competitions(method: &str, message_id: Uuid, data: Vec<CompetitionUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let data: Vec<Competition> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, competitions, competition_id, c)
    }).collect::<Result<Vec<Competition>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    publish::<SubType, Competition>(
        ws_conns, &data, SubType::Competition, None
    ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_series(method: &str, message_id: Uuid, data: Vec<ApiSeriesNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
        // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiSeriesNew::insert(&conn, data.clone())?;
   // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, data)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    let comp_and_series_ids = db::get_competition_ids_for_series(
        &conn, &data.iter().map(|s|s.series_id).dedup().collect()
    )?.into_iter().collect();
    publish::<SubType, ApiSeriesNew>(
        ws_conns, &data, SubType::Competition, Some(comp_and_series_ids)
    ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_series(method: &str, message_id: Uuid, data: Vec<SeriesUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Series> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, series, series_id, c)
    }).collect::<Result<Vec<Series>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    publish::<SubType, Series>(
        ws_conns, &out, SubType::Competition, None
    ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_matches(method: &str, message_id: Uuid, data: Vec<ApiMatchNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiMatchNew::insert(&conn, data.clone())?;
   // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, data)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    let series_ids: Vec<Uuid> = data.iter().map(|s| s.series_id).dedup().collect();
    let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?.into_iter().collect();
    publish::<SubType, ApiMatchNew>(
        ws_conns, &data, SubType::Competition, Some(comp_and_series_ids)
    ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_matches(method: &str, message_id: Uuid, data: Vec<MatchUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Match> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, matches, match_id, c)
    }).collect()})?;
    // TODO ideally would return response before awaiting publishing going out
    let series_ids: Vec<Uuid> = out.iter().map(|s| s.series_id).dedup().collect();
    let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?.into_iter().collect();
    publish::<SubType, Match>(
        ws_conns, &out, SubType::Competition, Some(comp_and_series_ids)
    ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_series_results(method: &str, message_id: Uuid, data: Vec<TeamSeriesResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let series_ids: Vec<Uuid> = data.iter().map(|x| x.series_id).collect();
    insert_exec!(&conn, schema::team_series_results::table, &data)?;
    let comp_to_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?.into_iter().collect();
    publish::<SubType, TeamSeriesResult>(ws_conns, &data, SubType::Competition, Some(comp_to_series_ids)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_series_results(method: &str, message_id: Uuid, data: Vec<TeamSeriesResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let series_ids: Vec<Uuid> = data.iter().map(|x| x.series_id).collect();
    let out: Vec<TeamSeriesResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, team_series_results, series_id, team_id, c)
    }).collect::<Result<Vec<TeamSeriesResult>, _>>()})?;
    let comp_to_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?.into_iter().collect();
    publish::<SubType, TeamSeriesResult>(ws_conns, &out, SubType::Competition, Some(comp_to_series_ids)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_match_results(method: &str, message_id: Uuid, data: Vec<TeamMatchResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let match_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    insert_exec!(&conn, schema::team_match_results::table, &data)?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish::<SubType, TeamMatchResult>(ws_conns, &data, SubType::Competition, Some(comp_id_map)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_match_results(method: &str, message_id: Uuid, data: Vec<TeamMatchResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let match_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    let out: Vec<TeamMatchResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, team_match_results, match_id, team_id, c)
    }).collect::<Result<Vec<TeamMatchResult>, _>>()})?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish::<SubType, TeamMatchResult>(ws_conns, &out, SubType::Competition, Some(comp_id_map)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_results(method: &str, message_id: Uuid, data: Vec<PlayerResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let match_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    insert_exec!(&conn, schema::player_results::table, &data)?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish::<SubType, PlayerResult>(ws_conns, &data, SubType::Competition, Some(comp_id_map)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_player_results(method: &str, message_id: Uuid, data: Vec<PlayerResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let match_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    let out: Vec<PlayerResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, player_results, match_id, player_id, c)
    }).collect()})?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish::<SubType, PlayerResult>(ws_conns, &out, SubType::Competition, Some(comp_id_map)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_teams(method: &str, message_id: Uuid, data: Vec<ApiTeam>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    ApiTeam::insert(conn, data.clone())?;
    publish::<SubType, ApiTeam>(ws_conns, &data, SubType::Team, None).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_teams(method: &str, message_id: Uuid, data: Vec<TeamUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Team> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update!(&conn, teams, team_id, c)
    }).collect()})?;
    publish::<SubType, Team>(ws_conns, &out, SubType::Team, None).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_players(method: &str, message_id: Uuid, data: Vec<ApiPlayer>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    ApiPlayer::insert(conn, data.clone())?;
    publish::<SubType, ApiPlayer>(ws_conns, &data, SubType::Team, None).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_players(method: &str, message_id: Uuid, data: Vec<PlayerUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Player> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update!(&conn, players, player_id, c)
    }).collect::<Result<Vec<Player>, _>>()})?;
    publish::<SubType, Player>(ws_conns, &out, SubType::Team, None).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_players(method: &str, message_id: Uuid, data: Vec<ApiTeamPlayer>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_team_players(&conn, &data)?;
    publish::<SubType, TeamPlayer>(ws_conns, &out, SubType::Team, None).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_names(method: &str, message_id: Uuid, data: Vec<ApiTeamNameNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_team_names(conn, data)?;
    publish::<SubType, TeamName>(ws_conns, &out, SubType::Team, None).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_names(method: &str, message_id: Uuid, data: Vec<ApiPlayerNameNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_player_names(&conn, &data)?;
    //let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    let team_id_map = db::get_player_ids_to_team_ids(
        &conn, &data.iter().map(|x|x.player_id).dedup().collect()
    )?.into_iter().collect();
    publish::<SubType, PlayerName>(ws_conns, &out, SubType::Team, Some(team_id_map)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_positions(method: &str, message_id: Uuid, data: Vec<ApiPlayerPositionNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_player_positions(&conn, &data)?;
    let team_id_map = db::get_player_ids_to_team_ids(
        &conn, &data.iter().map(|x|x.player_id).dedup().collect()
    )?.into_iter().collect();
    publish::<SubType, PlayerPosition>(ws_conns, &out, SubType::Team, Some(team_id_map)).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}
// TODO Prob need some deletions

pub async fn sub_competitions(method: &str, message_id: Uuid, data: SubCompetition, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
        // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = data.all{
        sub_all(&SubType::Competition, ws_user, toggle).await;
    }
    if let Some(competition_ids) = data.sub_competition_ids{
        sub(&SubType::Competition, ws_user, competition_ids.iter()).await;
    }
    if let Some(competition_ids) = data.unsub_competition_ids{
        unsub(&SubType::Competition, ws_user, competition_ids.iter()).await;
    }
    let subscription = ws_user.subscriptions.get_ez(&SubType::Competition);
    let rows = match subscription.all{
        true => {
            db::get_full_competitions(&conn, None)
        },
        false => {
            db::get_full_competitions(&conn, Some(subscription.ids.iter().collect()))
        }
    }?;
    let data = ApiCompetition::from_rows(rows);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_teams(method: &str, message_id: Uuid, data: SubTeam, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let toggle = data.toggle{
        sub_all(&SubType::Team, ws_user, toggle).await;
    }
    // Not supporting subbing to "some" teams yet
    // if let Some(competition_ids) = data.sub_competition_ids{
    //     sub(&SubType::Team, ws_user, competition_ids.iter()).await;
    // }
    // if let Some(competition_ids) = data.unsub_competition_ids{
    //     unsub(&SubType::Team, ws_user, competition_ids.iter()).await;
    // }
    let subscription = ws_user.subscriptions.get_ez(&SubType::Competition);
    let data = match subscription.all{
        true => {
            let team_out = db::get_all_teams(&conn).map(|rows| ApiTeam::from_rows(rows))?;
            let players_out = db::get_all_players(&conn).map(|rows| ApiPlayer::from_rows(rows))?;
            let team_players_out = db::get_all_team_players(&conn)?;
            ApiTeamsAndPlayers{teams: team_out, players: players_out, team_players: team_players_out}
        },
        false => ApiTeamsAndPlayers{teams: vec![], players: vec![], team_players: vec![]}
    };
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}