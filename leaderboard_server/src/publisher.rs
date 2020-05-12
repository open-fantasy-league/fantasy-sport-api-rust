use warp::ws;
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;
use serde::Serialize;
use crate::WSConnections_;
use diesel_utils::PgConn;
use crate::types::leaderboards::*;
use crate::subscriptions::{Subscription, SubType};
use crate::db;

pub trait Publishable {
    fn message_type<'a>() -> &'a str;
    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, conn: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>  where Self: Sized;
}

impl Publishable for Leaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard"
    }

    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, _: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>{
        Ok(
            match sub.all{
                // TODO anything nicer than iter->colelct?
                true => publishables.iter().collect(),
                false => {
                    match sub_type{
                        SubType::League => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.league_id)).collect()
                        },
                        SubType::Leaderboard => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.leaderboard_id)).collect()
                        }
                    }
                }
            }
        )
    }
}

impl Publishable for ApiLeaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard_detailed"
    }

    // Can commonise in a generic func between Leaderboard types
    // Would need to attach getters for league/leaderboard_id though
    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, _: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>{
        Ok(
            match sub.all{
                // TODO anything nicer than iter->colelct?
                true => publishables.iter().collect(),
                false => {
                    match sub_type{
                        SubType::League => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.league_id)).collect()
                        },
                        SubType::Leaderboard => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.leaderboard_id)).collect()
                        }
                    }
                }
            }
        )
    }
}

impl Publishable for Stat {
    fn message_type<'a>() -> &'a str {
        "stat"
    }

    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, conn: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>{
        Ok(
            match sub.all{
                // TODO anything nicer than iter->colelct?
                true => publishables.iter().collect(),
                false => {
                    match sub_type{
                        SubType::League => {
                            let id_map: HashMap<Uuid, Uuid> = db::get_league_ids_to_leaderboard_ids(
                                conn.unwrap(), publishables.iter().map(|s| s.leaderboard_id).collect()
                            )?.into_iter().collect();
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&id_map.get(&x.leaderboard_id).unwrap())).collect()
                    },
                    SubType::Leaderboard => publishables.iter()
                            .filter(|x| sub.ids.contains(&x.leaderboard_id)).collect()
                    }
                }
            }
        )
    }
}

pub async fn publish<T: Publishable + Serialize + std::fmt::Debug>(
    conn_opt: Option<PgConn>, ws_conns: &mut WSConnections_, publishables: &Vec<T>, sub_type: SubType
) -> Result<bool, BoxError>{
    // TODO This doesnt include team-names that were mutated by their name-timestamp being
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed_publishables: Vec<&T> = T::subscribed_publishables(publishables, wsconn.subscriptions.get(&sub_type), &sub_type, conn_opt.as_ref())?;
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