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

// Size for Self cannot be known at compile time.... :L
// #[async_trait]
// pub trait ServerInsertable{
//     async fn insert(conn: &PgConn, new: Vec<Self>) -> Result<bool, diesel::result::Error>;
//     fn comp_id_map_tup(
//         conn: PgConn,
//         me: &Vec<Self>,
//     ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error>;

// }

pub async fn insert_competitions(method: &str, message_id: Uuid, data: Vec<ApiCompetition>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
    // It's possible to just borrow it in db-insertion,
    // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
    ApiCompetition::insert(conn, data.clone()).await?;
    // TODO ideally would return response before awaiting publishing going out
    publish_for_comp::<ApiCompetition>(ws_conns, &data, data.iter().map(|c| (c.competition_id, c.competition_id)).collect()).await;
    println!("{:?}", &data);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_competitions(method: &str, message_id: Uuid, data: Vec<CompetitionUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let comps: Vec<Competition> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, competitions, competition_id, c)
    }).collect::<Result<Vec<Competition>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    publish_for_comp::<Competition>(
        ws_conns, &comps,
         comps.iter().map(|c| (c.competition_id, c.competition_id)).collect()
        ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, comps);
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
    )?;
    publish_for_comp::<ApiSeriesNew>(ws_conns, &data, comp_and_series_ids.into_iter().collect()).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_series(method: &str, message_id: Uuid, data: Vec<SeriesUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Series> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, series, series_id, c)
    }).collect::<Result<Vec<Series>, _>>()})?;
    // TODO ideally would return response before awaiting publishing going out
    publish_for_comp::<Series>(
        ws_conns, &out,
        out.iter().map(|c| (c.series_id, c.competition_id)).collect()
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
    let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    publish_for_comp::<ApiMatchNew>(ws_conns, &data, comp_and_series_ids.into_iter().collect()).await;
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
    let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    publish_for_comp::<Match>(
        ws_conns, &out,
        comp_and_series_ids.into_iter().collect()
        ).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_series_results(method: &str, message_id: Uuid, data: Vec<TeamSeriesResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let series_ids: Vec<Uuid> = data.iter().map(|x| x.series_id).collect();
    insert_exec!(&conn, schema::team_series_results::table, &data)?;
    let competition_n_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    let comp_to_series_ids: HashMap<Uuid, Uuid> = competition_n_series_ids.into_iter().collect();
    publish_for_comp::<TeamSeriesResult>(ws_conns, &data, comp_to_series_ids).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_team_series_results(method: &str, message_id: Uuid, data: Vec<TeamSeriesResultUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let series_ids: Vec<Uuid> = data.iter().map(|x| x.series_id).collect();
    let out: Vec<TeamSeriesResult> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update_2pkey!(&conn, team_series_results, series_id, team_id, c)
    }).collect::<Result<Vec<TeamSeriesResult>, _>>()})?;
    let competition_n_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;
    let comp_to_series_ids: HashMap<Uuid, Uuid> = competition_n_series_ids.into_iter().collect();
    publish_for_comp::<TeamSeriesResult>(ws_conns, &out, comp_to_series_ids).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_match_results(method: &str, message_id: Uuid, data: Vec<TeamMatchResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let match_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    insert_exec!(&conn, schema::team_match_results::table, &data)?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish_for_comp::<TeamMatchResult>(ws_conns, &data, comp_id_map).await;
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
    publish_for_comp::<TeamMatchResult>(ws_conns, &out, comp_id_map).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_results(method: &str, message_id: Uuid, data: Vec<PlayerResult>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let match_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    insert_exec!(&conn, schema::player_results::table, &data)?;
    let comp_id_map: HashMap<Uuid, Uuid> = db::get_competition_ids_for_matches(&conn, &match_ids)?.into_iter().collect();
    publish_for_comp::<PlayerResult>(ws_conns, &data, comp_id_map).await;
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
    publish_for_comp::<PlayerResult>(ws_conns, &out, comp_id_map).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_teams(method: &str, message_id: Uuid, data: Vec<ApiTeam>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    ApiTeam::insert(conn, data.clone())?;
    publish_for_teams::<ApiTeam>(ws_conns, &data).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_teams(method: &str, message_id: Uuid, data: Vec<TeamUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Team> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update!(&conn, teams, team_id, c)
    }).collect()})?;
    publish_for_teams::<Team>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_players(method: &str, message_id: Uuid, data: Vec<ApiPlayer>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    ApiPlayer::insert(conn, data.clone())?;
    publish_for_teams::<ApiPlayer>(ws_conns, &data).await;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_players(method: &str, message_id: Uuid, data: Vec<PlayerUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Player> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
            update!(&conn, players, player_id, c)
    }).collect::<Result<Vec<Player>, _>>()})?;
    publish_for_teams::<Player>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_players(method: &str, message_id: Uuid, data: Vec<ApiTeamPlayer>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_team_players(conn, data).await?;
    publish_for_teams::<TeamPlayer>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_team_names(method: &str, message_id: Uuid, data: Vec<ApiTeamNameNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_team_names(conn, data).await?;
    publish_for_teams::<TeamName>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_names(method: &str, message_id: Uuid, data: Vec<ApiPlayerNameNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_player_names(conn, data).await?;
    publish_for_teams::<PlayerName>(ws_conns, &out).await;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_player_positions(method: &str, message_id: Uuid, data: Vec<ApiPlayerPositionNew>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out = db::insert_player_positions(conn, data).await?;
    publish_for_teams::<PlayerPosition>(ws_conns, &out).await;
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
        sub_to_all_competitions(ws_user, toggle).await;
    }
    if let Some(competition_ids) = data.sub_competition_ids{
        sub_to_competitions(ws_user, competition_ids.iter()).await;
    }
    if let Some(competition_ids) = data.unsub_competition_ids{
        unsub_to_competitions(ws_user, competition_ids.iter()).await;
    }
    let all_competitions = db::get_all_competitions(&conn)?;
    let subscribed_comps: Vec<&Competition> = subscribed_comps::<Competition>(&ws_user.subscriptions, &all_competitions);
    let comp_rows = db::get_full_competitions(
        &conn, subscribed_comps.iter().map(|x| x.competition_id).collect()
    )?;
    let data = ApiCompetition::from_rows(comp_rows);
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_teams(method: &str, message_id: Uuid, data: SubTeam, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
        let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    sub_to_teams(ws_user, data.toggle).await;

    // TODO kind of weird code-duping in match arms.
    // would be nice to just return data, but either have to make an enum, or Box<dyn Serialize?> (couldnt even get that to work)
    let resp = match data.toggle{
        true => {
            let team_out = db::get_all_teams(&conn).map(|rows| ApiTeam::from_rows(rows))?;
            let players_out = db::get_all_players(&conn).map(|rows| ApiPlayer::from_rows(rows))?;
            let team_players_out = db::get_all_team_players(&conn)?;
            let data = ApiTeamsAndPlayers{teams: team_out, players: players_out, team_players: team_players_out};
            let resp_msg = WSMsgOut::resp(message_id, method, data);
            serde_json::to_string(&resp_msg).map_err(|e| e.into())
        },
        false => {
            let data = json!({});
            let resp_msg = WSMsgOut::resp(message_id, method, data);
            serde_json::to_string(&resp_msg).map_err(|e| e.into())
        }
    };
    resp
}

// Nice idea but Deserilize complains about different liftimes
// TODO Work out why and how to fix
// pub async fn insert_generic<'a, T: ServerInsertable + std::fmt::Debug + Deserialize<'a> + Clone + Publishable + Serialize>(method: &str, message_id: Uuid, data: Vec<Pick>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
//     // //     // Need to clone comps, so that can still publish it, after has been "consumed" adding to db.
//     // It's possible to just borrow it in db-insertion,
//     // but it leads to having to specify lifetimes on nearly EVERYTHING. Not worth the hassle unless need perf
//     T::insert(&conn, data.clone()).await?;
//    // let comps: Vec<Competition> = insert!(&conn, schema::competitions::table, data)?;
//     // assume anything upserted the user wants to subscribe to
//     // TODO ideally would return response before awaiting publishing going out
//     let comp_id_map_tup = T::comp_id_map_tup(
//         conn, &data
//     )?;
//     publish_for_comp::<T>(ws_conns, &data, comp_id_map_tup.into_iter().collect()).await;
//     let resp_msg = WSMsgOut::resp(message_id, method, data);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }