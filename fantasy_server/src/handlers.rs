use warp_ws_server::*;
use crate::{db, WSConnections_};
use uuid::Uuid;
#[macro_use]
use diesel_utils::*;
use crate::schema::{self,*};
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use crate::types::{leagues::*};
use crate::subscriptions;

pub async fn insert_leagues(req: WSReq<'_>, conn: PgConn, _: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<League> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let leagues: Vec<League> = insert!(&conn, leagues::table, deserialized)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    //publish_leagues(ws_conns, &leagues).await;
    println!("{:?}", &leagues);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub async fn update_leagues(req: WSReq<'_>, conn: PgConn, _: &mut WSConnections_) -> Result<String, BoxError>{
    let deserialized: Vec<LeagueUpdate> = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let leagues: Vec<League> = conn.build_transaction().run(|| {
        deserialized.iter().map(|c| {
        update!(&conn, leagues, league_id, c)
    }).collect::<Result<Vec<League>, _>>()})?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    //publish_leagues(ws_conns, &leagues).await;
    println!("{:?}", &leagues);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

// Could prob commonise the sub-methods into ws-server
pub async fn sub_leagues(req: WSReq<'_>, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid) -> Result<String, BoxError>{
    let deserialized: ApiSubLeagues = serde_json::from_value(req.data)?;
    // let ws_user = ws_conns.lock().await.get_mut(&user_ws_id).ok_or("Webscoket gone away")?;
    // why does this need splitting into two lines?
    // ANd is it holding the lock for this whole scope? doesnt need to
    let mut hmmmm = ws_conns.lock().await;
    let ws_user = hmmmm.get_mut(&user_ws_id).ok_or("Websocket gone away")?;
    if let Some(toggle) = deserialized.all{
        sub_to_all_leagues(ws_user, toggle).await;
    }
    else if let Some(ids) = deserialized.league_ids{
        sub_to_leagues(ws_user, ids.iter()).await;
    }
    else{
        return Err(Box::new(InvalidRequestError{description: String::from("sub_competitions must specify either 'all' or 'competition_ids'")}))
    }
    let all = db::get_all_leagues(&conn)?;
    let subscribed_to: Vec<&League> = subscriptions::subscribed_leagues::<League>(&ws_user.subscriptions, &all);
    let comp_rows = db::get_full_leagues(
        &conn, subscribed_to.iter().map(|x| x.competition_id).collect()
    )?;
    let data = ApiLeague::from_rows(comp_rows);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

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
//     let subscribed_comps: Vec<&Competition> = subscribed_comps::<Competition>(&ws_user.subscriptions, &all_competitions);
//     let comp_rows = db::get_full_competitions(
//         &conn, subscribed_comps.iter().map(|x| x.competition_id).collect()
//     )?;
//     let data = ApiCompetition::from_rows(comp_rows);
//     let resp_msg = WSMsgOut::resp(req.message_id, req.method, data);
//     serde_json::to_string(&resp_msg).map_err(|e| e.into())
// }