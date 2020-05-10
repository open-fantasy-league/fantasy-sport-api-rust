use warp_ws_server::*;
use crate::{db, WSConnections_};
use uuid::Uuid;
#[macro_use]
use diesel_utils::*;
use crate::schema::{self,*};
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use crate::types::{leagues::*, users::*, drafts::*, fantasy_teams::*};
use crate::subscriptions::*;
use crate::publisher::*;

pub async fn insert_leagues(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<League> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    // TODO reduce the ridiculousness of the Values type
    //let leagues: Vec<League> = db::insert::<League, leagues::table, diesel::insertable::OwnedBatchInsert<diesel::query_builder::ValuesClause<(_, _, _, _, _, _, _, _, _), schema::leagues::table>, schema::leagues::table>>(req, conn, leagues::table)?;
    let leagues: Vec<League> = insert!(&conn, leagues::table, deserialized)?;
    println!("{:?}", &leagues);
    publish_for_leagues::<League>(
        conn, ws_conns, &leagues,
    ).await?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_leagues(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<LeagueUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let leagues: Vec<League> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, leagues, league_id, c)
    }).collect()})?;
    publish_for_leagues::<League>(
        conn, ws_conns, &leagues,
    ).await?;
    println!("{:?}", &leagues);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_periods(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<Period> = serde_json::from_value(req.data)?;
    println!("{:#?}", &deserialized);
    let out: Vec<Period> = insert!(&conn, periods::table, deserialized)?;
    println!("{:#?}", &out);
    publish_for_leagues::<Period>(
        conn, ws_conns, &out,
    ).await?;
    println!("postpublish");
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_periods(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<PeriodUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let periods: Vec<Period> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, periods, period_id, c)
    }).collect()})?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    //publish_leagues(ws_conns, &leagues).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, periods);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_stat_multipliers(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<StatMultiplier> = serde_json::from_value(req.data)?;
    let out: Vec<StatMultiplier> = insert!(&conn, stat_multipliers::table, deserialized)?;
    publish_for_leagues::<StatMultiplier>(
        conn, ws_conns, &out,
    ).await?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_stat_multipliers(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<StatMultiplierUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<StatMultiplier> = conn.build_transaction().run(|| {
        deserialized.into_iter().map(|c| {
        // TODO using 2pkey, but is it legit that cannot change name once set?
        // maybe should have a uuid pkey
        // this clone a bit hacky, the macro was originally just doing UUIDs which implement copy (string name doesnt)
        update_2pkey!(&conn, stat_multipliers, league_id, name, c.clone())
    }).collect()})?;
    publish_for_leagues::<StatMultiplier>(
        conn, ws_conns, &out,
    ).await?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_external_users(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ExternalUser> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    // TODO reduce the ridiculousness of the Values type
    //let external_users: Vec<League> = db::insert::<League, external_users::table, diesel::insertable::OwnedBatchInsert<diesel::query_builder::ValuesClause<(_, _, _, _, _, _, _, _, _), schema::external_users::table>, schema::external_users::table>>(req, conn, external_users::table)?;
    let external_users: Vec<ExternalUser> = insert!(&conn, external_users::table, deserialized)?;
    println!("{:?}", &external_users);
    // TODO external user publishing
    // publish_for_leagues::<League>(
    //     conn, ws_conns, &leagues,
    // ).await?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, external_users);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_external_users(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<ExternalUserUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let external_users: Vec<ExternalUser> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, external_users, external_user_id, c)
    }).collect()})?;
    // publish_for_leagues::<League>(
    //     conn, ws_conns, &leagues,
    // ).await?;
    println!("{:?}", &external_users);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, external_users);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_draft_queues(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<DraftQueue> = serde_json::from_value(req.data)?;
    let out: Vec<DraftQueue> = insert!(&conn, draft_queues::table, deserialized)?;
    // TODO do draft-queues even want publishing to anyone except caller (person's queue should be private)
    //let id_map = db::get_league_ids_for_draft_queues(&conn, &series_ids)?;
    // publish_for_leagues::<DraftQueue>(
    //     ws_conns, &out,
    //     out.iter().map(|c| (c.draft, c.league_id)).collect()
    // ).await;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_draft_queues(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<DraftQueue> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<DraftQueue> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, draft_queues, fantasy_team_id, c)
    }).collect()})?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// Deliberate no insert_draft_choice as system creates
// We just update when pick has been made
// TODO hmmm shouldnt draft-queue also be system-generated?
//actually remove this? draft-choice should be updated by pick
pub async fn update_draft_choices(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<DraftChoiceUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<DraftChoice> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, draft_choices, draft_choice_id, c)
    }).collect()})?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_picks(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<Pick> = serde_json::from_value(req.data)?;
    let out: Vec<Pick> = insert!(&conn, picks::table, deserialized)?;
    // TODO do draft-queues even want publishing to anyone except caller (person's queue should be private)
    publish_for_drafts::<Pick>(conn, ws_conns, &out).await?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_picks(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<PickUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<Pick> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, picks, pick_id, c)
    }).collect()})?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn insert_fantasy_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<FantasyTeam> = serde_json::from_value(req.data)?;
    let out: Vec<FantasyTeam> = insert!(&conn, fantasy_teams::table, deserialized)?;
    publish_for_leagues::<FantasyTeam>(
        conn, ws_conns, &out,
    ).await?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_fantasy_teams(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<FantasyTeamUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let out: Vec<FantasyTeam> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, fantasy_teams, fantasy_team_id, c)
    }).collect()})?;
    // TODO what's the subscription for this?
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, out);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// impl From<serde_json::error::Error> for std::error::Error {
//     fn from(item: serde_json::error::Error) -> Self {
//         std::error::Error
//     }
// }

// Could prob commonise the sub-methods into ws-server
pub async fn sub_leagues(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    // let deserialized: ApiSubLeagues = serde_json::from_value(req.data).map_err(|e: serde_json::error::Error|
    //      serde::ser::Error::custom(format!("{}. line: {}, column: {}", e.to_string(), e.line(), e.column())
    //     ))?;
    let deserialized: ApiSubLeagues = serde_json::from_value(req.data)?;
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = deserialized.all{
        sub_to_all_leagues(ws_user, toggle).await;
    }
    if let Some(ids) = deserialized.sub_league_ids{
        sub_to_leagues(ws_user, ids.iter()).await;
    }
    if let Some(ids) = deserialized.unsub_league_ids{
        unsub_to_leagues(ws_user, ids.iter()).await;
    }
    let all = schema::leagues::table.load(&conn)?;
    let subscribed_to: Vec<&League> = subscribed_leagues::<League>(&conn, &ws_user.subscriptions, &all)?;
    let data = db::get_full_leagues(
        &conn, subscribed_to.iter().map(|x| x.competition_id).collect()
    )?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_drafts(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let deserialized: ApiSubDrafts = serde_json::from_value(req.data)?;
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = deserialized.all{
        sub_to_all_drafts(ws_user, toggle).await;
    }
    if let Some(ids) = deserialized.sub_draft_ids{
        sub_to_drafts(ws_user, ids.iter()).await;
    }
    if let Some(ids) = deserialized.unsub_draft_ids{
        unsub_to_drafts(ws_user, ids.iter()).await;
    }
    let all = schema::leagues::table.load(&conn)?;
    let subscribed_to: Vec<&League> = subscribed_leagues::<League>(&conn, &ws_user.subscriptions, &all)?;
    let data = db::get_full_leagues(
        &conn, subscribed_to.iter().map(|x| x.competition_id).collect()
    )?;
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn sub_external_users(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let deserialized: ApiSubExternalUsers = serde_json::from_value(req.data)?;
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    println!("{:?}", &deserialized);
    sub_to_external_users(ws_user, deserialized.toggle).await;
    match deserialized.toggle{
        true => {
            let t: (Vec<ExternalUser>, Vec<Commissioner>) = db::get_users(&conn)?;
            let data = UsersAndCommissioners{users: t.0, commissioners: t.1};
            let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
            serde_json::to_string(&resp_msg).map_err(|e| e.into())
        },
        false => {
            let data = serde_json::json!({});
            let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
            serde_json::to_string(&resp_msg).map_err(|e| e.into())
        }
    }
}