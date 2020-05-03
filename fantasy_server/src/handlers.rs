use warp_ws_server::*;
use crate::{db, WSConnections_};
use uuid::Uuid;

pub fn insert_leagues(req: WSReq, conn: PgConn, _: &mut WSConnections_, _: Uuid) -> Result<String, BoxError>{
    let deserialized = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let leagues = db::insert_leagues(&conn, deserialized)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    //publish_leagues(ws_conns, &leagues).await;
    println!("{:?}", &leagues);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, leagues);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}

pub fn update_league(req: WSReq, conn: PgConn, _: &mut WSConnections_, _: Uuid) -> Result<String, BoxError>{
    let deserialized = serde_json::from_value(req.data)?;
    println!("{:?}", &deserialized);
    let league = db::update_league(&conn, deserialized)?;
    // assume anything upserted the user wants to subscribe to
    // TODO ideally would return response before awaiting publishing going out
    //publish_leagues(ws_conns, &leagues).await;
    println!("{:?}", &league);
    let resp_msg = WSMsgOut::resp(req.message_id, req.method, league);
    serde_json::to_string(&resp_msg).map_err(|e| e.into())
}