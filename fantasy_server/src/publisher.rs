use warp::ws;
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;
use serde::Serialize;
use crate::WSConnections_;

pub trait Publishable {
    fn message_type<'a>() -> &'a str;
    fn get_hierarchy_id(&self) -> Uuid;
    fn subscription_id(&self) -> Uuid;
}


pub async fn publish_for_leagues<T: Publishable + Serialize + std::fmt::Debug>(ws_conns: &mut WSConnections_, publishables: &Vec<T>, id_map: HashMap<Uuid, Uuid>){
    // TODO This doesnt include team-names that were mutated by their name-timestamp being 
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed_publishables: Vec<&T> = publishables.iter()
            .filter(|x| wsconn.subscriptions.leagues.contains(&id_map.get(&x.get_hierarchy_id()).unwrap())).collect();
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
}

pub async fn publish_for_drafts<T: Publishable + Serialize + std::fmt::Debug>(ws_conns: &mut WSConnections_, publishables: &Vec<T>, id_map: HashMap<Uuid, Uuid>){
    // TODO This doesnt include team-names that were mutated by their name-timestamp being 
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed_publishables: Vec<&T> = publishables.iter()
            .filter(|x| wsconn.subscriptions.drafts.contains(&id_map.get(&x.get_hierarchy_id()).unwrap())).collect();
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
}