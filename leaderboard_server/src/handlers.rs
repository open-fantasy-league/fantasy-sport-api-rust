use crate::schema::{self, *};
use crate::WSConnections_;
use uuid::Uuid;
use crate::types::leaderboards::*;
use warp_ws_server::{WSMsgOut, BoxError};
use crate::subscriptions::*;
use crate::db;
use crate::messages::*;
use diesel_utils::*;
use crate::publisher::*;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;

pub async fn sub_leagues(method: &str, message_id: Uuid, data: SubLeague, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = data.all{
        sub_to_all_leagues(ws_user, toggle).await;
    }
    if let Some(ids) = data.sub_league_ids{
        sub_to_leagues(ws_user, ids.iter()).await;
    }
    if let Some(ids) = data.unsub_league_ids{
        unsub_to_leagues(ws_user, ids.iter()).await;
    }
    let subscriptions = &ws_user.subscriptions;
    let data = match subscriptions.all_leagues{
        true => {
            db::get_full_leagues(&conn, None, None)
        },
        false => {
            db::get_full_leagues(&conn, Some(subscriptions.leagues.iter().collect()), None)
        }
    }?;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_leaderboards(method: &str, message_id: Uuid, data: SubLeaderboard, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = data.all{
        sub_to_all_leaderboards(ws_user, toggle).await;
    }
    if let Some(ids) = data.sub_leaderboard_ids{
        sub_to_leaderboards(ws_user, ids.iter()).await;
    }
    if let Some(ids) = data.unsub_leaderboard_ids{
        unsub_to_leaderboards(ws_user, ids.iter()).await;
    }
    let subscriptions = &ws_user.subscriptions;
    let data = match subscriptions.all_leaderboards{
        true => {
            db::get_full_leagues(&conn, None, None)
        },
        false => {
            db::get_full_leagues(&conn, None, Some(subscriptions.leaderboards.iter().collect()))
        }
    }?;
    let resp_msg = WSMsgOut::resp(message_id, method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_leaderboards(method: &str, message_id: Uuid, data: Vec<Leaderboard>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Leaderboard> = insert!(&conn, leaderboards::table, data)?;
    // publish_for_leagues::<Leaderboard>(
    //     None, ws_conns, &out,
    // ).await?;
    publish_for_leaderboards::<Leaderboard>(
        None, ws_conns, &out,
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_leaderboards(method: &str, message_id: Uuid, data: Vec<LeaderboardUpdate>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let out: Vec<Leaderboard> = conn.build_transaction().run(|| {
        data.iter().map(|c| {
        update!(&conn, leaderboards, leaderboard_id, c)
    }).collect()})?;
    // publish_for_leagues::<Leaderboard>(
    //     None, ws_conns, &out,
    // ).await?;
    publish_for_leaderboards::<Leaderboard>(
        None, ws_conns, &out,
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_stats(method: &str, message_id: Uuid, data: Vec<Stat>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    // TODO reduce the ridiculousness of the Values type
    let out: Vec<Stat> = insert!(&conn, stats::table, data)?;
    // publish_for_leagues::<Player>(
    //     Some(&conn), ws_conns, &out,
    // ).await?;
    publish_for_leaderboards::<Stat>(
        None, ws_conns, &out,
    ).await?;
    let resp_msg = WSMsgOut::resp(message_id, method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}