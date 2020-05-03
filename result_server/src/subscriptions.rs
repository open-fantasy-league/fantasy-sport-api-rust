use crate::WSConnection_;
use crate::models;
use uuid::Uuid;
use std::collections::HashSet;
pub struct Subscriptions{
    pub teams: bool,
    pub competitions: HashSet<Uuid>,
    pub all_competitions: bool
}

impl warp_ws_server::Subscriptions for Subscriptions{
    fn new() -> Subscriptions {
        Subscriptions{teams: false, competitions: HashSet::new(), all_competitions: false}
    }
}

pub async fn sub_to_competitions<'a, T: Iterator<Item = &'a Uuid>>(ws_user: &mut WSConnection_, competition_ids: T){
    competition_ids.for_each(|cid| {
        println!("Adding subscription {}", cid); ws_user.subscriptions.competitions.insert(*cid);
    });
}

pub async fn sub_to_all_competitions(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.all_competitions = toggle
}

pub async fn sub_to_teams(ws_user: &mut WSConnection_, toggle: bool){
    ws_user.subscriptions.teams = toggle
}

// TODO make generic with series and matches, T and closure for competition_id? or trait for HasCompetition?
pub fn subscribed_comps<'a>(subscriptions: &Subscriptions, all_comps: &'a Vec<models::Competition>) -> Vec<&'a models::Competition>{
    match subscriptions.all_competitions{
        // turn from &Vec<models::Competition> into Vec<&models::Competition>
        // Passing in &Vec to func, so that publish and send response can 'share' competition. i.e. publishing doesnt consume it.
        // However is probably simpler to set up so can just clone it, and this func mvoes Vec, rather than ref
        true => all_comps.iter().collect(),
        false => {
            all_comps.iter()
            .filter(|c| subscriptions.competitions.contains(&c.competition_id))
            .collect()
        }
    }
}