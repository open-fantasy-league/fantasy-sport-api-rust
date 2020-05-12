use uuid::Uuid;
use crate::WSConnection_;
use serde::Deserialize;
use crate::publisher::Publishable;
use diesel_utils::PgConn;
use std::collections::{HashSet, HashMap};
use warp_ws_server::BoxError;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum SubType{
    League,
    Leaderboard
}

// pub struct Subscription{
//     pub ids: HashSet<Uuid>,
//     pub all: bool
// }

// impl Subscription{
//     fn new() -> Self {
//         Self{ids: HashSet::new(), all: false}
//     }
// }

// //pub type Subscriptions<'a> = HashMap<&'a str, Subscription>;
// pub struct Subscriptions{
//     pub inner: HashMap<SubType, Subscription>,
// }

// impl Subscriptions{

//     pub fn get(&mut self, sub_type: &SubType) -> &mut Subscription{
//         self.inner.get_mut(sub_type).unwrap()
//     }
// }

impl warp_ws_server::Subscriptions for Subscriptions{
    fn new() -> Subscriptions {
        let mut inner = HashMap::new();
        inner.insert(SubType::League, Subscription::new());
        inner.insert(SubType::Leaderboard, Subscription::new());
        Subscriptions{inner}
    }
}

// impl warp_ws_server::Subscriptions for Subscriptions{
//     fn new() -> Subscriptions {
//         Subscriptions{leagues: HashSet::new(), leaderboards: HashSet::new(), all_leagues: false, all_leaderboards: false}
//     }
// }


pub async fn sub_to_leagues<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.inner.get_mut(&SubType::League).unwrap().ids.insert(*id);
    });
}

pub async fn unsub_to_leagues<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    // TODO failure handling, does it panic?
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.inner.get_mut(&SubType::League).unwrap().ids.remove(id);
    });
}

pub async fn sub_to_leaderboards<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.inner.get_mut(&SubType::Leaderboard).unwrap().ids.insert(*id);
    });
}

pub async fn unsub_to_leaderboards<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    // TODO failure handling, does it panic?
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.inner.get_mut(&SubType::Leaderboard).unwrap().ids.remove(id);
    });
}

pub async fn sub_to_all_leagues(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.inner.get_mut(&SubType::League).unwrap().all = toggle;
}

pub async fn sub_to_all_leaderboards(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.inner.get_mut(&SubType::Leaderboard).unwrap().all = toggle;
}