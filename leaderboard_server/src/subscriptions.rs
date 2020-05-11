use uuid::Uuid;
use crate::WSConnection_;
use serde::Deserialize;
use crate::publisher::Publishable;
use diesel_utils::PgConn;
use std::collections::{HashSet, HashMap};
use warp_ws_server::BoxError;

// Maybe split up subscriptions into a hashmap is better for commonising?
pub struct Subscriptions{
    pub leagues: HashSet<Uuid>,
    pub leaderboards: HashSet<Uuid>,
    pub all_leagues: bool,
    pub all_leaderboards: bool
}

impl warp_ws_server::Subscriptions for Subscriptions{
    fn new() -> Subscriptions {
        Subscriptions{leagues: HashSet::new(), leaderboards: HashSet::new(), all_leagues: false, all_leaderboards: false}
    }
}


pub async fn sub_to_leagues<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.leagues.insert(*id);
    });
}

pub async fn unsub_to_leagues<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    // TODO failure handling, does it panic?
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.leagues.remove(id);
    });
}

pub async fn sub_to_leaderboards<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.leaderboards.insert(*id);
    });
}

pub async fn unsub_to_leaderboards<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    // TODO failure handling, does it panic?
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.leaderboards.remove(id);
    });
}

pub async fn sub_to_all_leagues(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.all_leagues = toggle
}

pub async fn sub_to_all_leaderboards(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.all_leaderboards = toggle
}