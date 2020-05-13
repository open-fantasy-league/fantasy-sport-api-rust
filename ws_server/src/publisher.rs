use crate::subscriptions::{Subscription, GetEz};
use crate::{BoxError, PgConn, WSMsgOut, WSConnections};
use serde::Serialize;
use warp::ws;
use uuid::Uuid;
use std::collections::HashMap;

pub trait Publishable<CustomSubType> {
    fn message_type<'a>() -> &'a str;

    fn subscribed_publishables<'b>(
        publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &CustomSubType, id_map_opt: &Option<HashMap<Uuid, Uuid>>
    ) -> Vec<&'b Self> where Self: Sized{
        match sub.all{
            // TODO anything nicer than iter->colelct?
            true => publishables.iter().collect(),
            false => {
                Self::partial_subscribed_publishables(publishables, sub, sub_type, id_map_opt)
            }
        }
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &CustomSubType, id_map_opt: &Option<HashMap<Uuid, Uuid>>
    ) -> Vec<&'b Self> where Self: Sized;
}


pub async fn publish<CustomSubType: std::cmp::Eq + std::hash::Hash, T: Publishable<CustomSubType> + Serialize + std::fmt::Debug>(
    ws_conns: &mut WSConnections<CustomSubType>, publishables: &Vec<T>, sub_type: CustomSubType, id_map_opt: Option<HashMap<Uuid, Uuid>>
) -> Result<bool, BoxError>{
    // TODO COuld be optimised with some kind of caching for same messages to different users
    // (i.e. everyone subscribed to `all`, will definitely get the same message)
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed_publishables: Vec<&T> = T::subscribed_publishables(publishables, wsconn.subscriptions.get_ez(&sub_type), &sub_type, &id_map_opt);
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