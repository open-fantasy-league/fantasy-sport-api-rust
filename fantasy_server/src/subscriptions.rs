use uuid::Uuid;
use std::collections::HashSet;
use crate::WSConnection_;
use serde::Deserialize;
use crate::publisher::Publishable;

// Maybe split up subscriptions into a hashmap is better for commonising?
pub struct Subscriptions{
    pub external_users: bool,
    pub leagues: HashSet<Uuid>,
    pub drafts: HashSet<Uuid>,
    pub all_leagues: bool,
    pub all_drafts: bool
}

impl warp_ws_server::Subscriptions for Subscriptions{
    fn new() -> Subscriptions {
        Subscriptions{external_users: false, leagues: HashSet::new(), drafts: HashSet::new(), all_leagues: false, all_drafts: false}
    }
}

pub trait Subscribable{
    fn subscription_id(&self) -> Uuid;
}

#[derive(Deserialize, Debug)]
pub struct ApiSubExternalUsers{
    pub toggle: bool,
}

#[derive(Deserialize, Debug)]
pub struct ApiSubDrafts{
    pub sub_draft_ids: Option<Vec<Uuid>>,
    pub unsub_draft_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>
}

#[derive(Deserialize, Debug)]
pub struct ApiSubLeagues{
    pub sub_league_ids: Option<Vec<Uuid>>,
    pub unsub_league_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>
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

pub async fn sub_to_drafts<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.drafts.insert(*id);
    });
}

pub async fn unsub_to_drafts<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, ids: T){
    // TODO failure handling, does it panic?
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.drafts.remove(id);
    });
}

pub async fn sub_to_all_leagues(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.all_leagues = toggle
}

pub async fn sub_to_all_drafts(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.all_drafts = toggle
}

pub async fn sub_to_external_users(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.external_users = toggle
}


// TODO make generic with series and matches, T and closure for competition_id? or trait for HasCompetition?
pub fn subscribed_leagues<'a, T: Publishable>(subscriptions: &Subscriptions, all: &'a Vec<T>) -> Vec<&'a T>{
    match subscriptions.all_leagues{
        // turn from &Vec<Competition> into Vec<&Competition>
        // Passing in &Vec to func, so that publish and send response can 'share' competition. i.e. publishing doesnt consume it.
        // However is probably simpler to set up so can just clone it, and this func mvoes Vec, rather than ref
        true => all.iter().collect(),
        false => {
            all.iter()
            .filter(|c| subscriptions.leagues.contains(&c.subscription_id()))
            .collect()
        }
    }
}

pub fn subscribed_drafts<'a, T: Publishable>(subscriptions: &Subscriptions, all: &'a Vec<T>) -> Vec<&'a T>{
    match subscriptions.all_drafts{
        // turn from &Vec<Competition> into Vec<&Competition>
        // Passing in &Vec to func, so that publish and send response can 'share' competition. i.e. publishing doesnt consume it.
        // However is probably simpler to set up so can just clone it, and this func mvoes Vec, rather than ref
        true => all.iter().collect(),
        false => {
            all.iter()
            .filter(|c| subscriptions.drafts.contains(&c.subscription_id()))
            .collect()
        }
    }
}