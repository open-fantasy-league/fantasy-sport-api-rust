use warp::ws;
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;
use serde::Serialize;
use crate::WSConnections_;
use diesel_utils::PgConn;

pub trait Publishable {
    fn message_type<'a>() -> &'a str;
    fn subscription_map_key(&self) -> Uuid;
    fn subscription_id_map(conn: Option<&PgConn>, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError> where Self: Sized;
}

//let comp_and_series_ids = db::get_competition_ids_for_series(&conn, &series_ids)?;


pub async fn publish_for_leagues<T: Publishable + Serialize + std::fmt::Debug>(conn_opt: Option<PgConn>, ws_conns: &mut WSConnections_, publishables: &Vec<T>) -> Result<bool, BoxError>{
    // TODO This doesnt include team-names that were mutated by their name-timestamp being
    let id_map: HashMap<Uuid, Uuid> = T::subscription_id_map(conn_opt.as_ref(), publishables)?; 
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        println!("publish_f_leagues");
        let subscribed_publishables: Vec<&T> = publishables.iter()
            .filter(|x| wsconn.subscriptions.leagues.contains(&id_map.get(&x.subscription_map_key()).unwrap())).collect();
        println!("publish_f_leagues2");
        let push_msg = WSMsgOut::push(T::message_type(), subscribed_publishables);
        let subscribed_json_r = serde_json::to_string(&push_msg);
        match subscribed_json_r.as_ref(){
            Ok(subscribed_json) => {
                if let Err(publish) = wsconn.tx.send(Ok(ws::Message::text(subscribed_json))){
                    println!("Error publishing update {:?} to {} : {}", &subscribed_json, uid, &publish)
                };
            },
            Err(_) => println!("Error json serializing publisher update {:?} to {}", &subscribed_json_r, uid)
        };
    };
    Ok(true)
}

pub async fn publish_for_drafts<T: Publishable + Serialize + std::fmt::Debug>(conn_opt: Option<PgConn>, ws_conns: &mut WSConnections_, publishables: &Vec<T>) -> Result<bool, BoxError>{
    // TODO This doesnt include team-names that were mutated by their name-timestamp being
    let id_map: HashMap<Uuid, Uuid> = T::subscription_id_map(conn_opt.as_ref(), publishables)?;
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed_publishables: Vec<&T> = publishables.iter()
            .filter(|x| wsconn.subscriptions.drafts.contains(&id_map.get(&x.subscription_map_key()).unwrap())).collect();
        let push_msg = WSMsgOut::push(T::message_type(), subscribed_publishables);
        let subscribed_json_r = serde_json::to_string(&push_msg);
        match subscribed_json_r.as_ref(){
            Ok(subscribed_json) => {
                if let Err(publish) = wsconn.tx.send(Ok(ws::Message::text(subscribed_json))){
                    println!("Error publishing update {:?} to {} : {}", &subscribed_json, uid, &publish)
                };
            },
            Err(_) => println!("Error json serializing publisher update {:?} to {}", &subscribed_json_r, uid)
        };
    };
    Ok(true)
}