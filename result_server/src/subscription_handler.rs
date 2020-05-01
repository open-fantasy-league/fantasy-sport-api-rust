use crate::WsConnections;
use uuid::Uuid;
use std::collections::HashSet;
pub struct Subscriptions{
    pub teams: bool,
    pub players: bool,
    pub competitions: HashSet<Uuid>
}

impl Subscriptions{
    pub fn new() -> Subscriptions {
        Subscriptions{teams: false, players: false, competitions: HashSet::new()}
    }
}

pub async fn sub_to_competitions<'a, T: Iterator<Item = &'a Uuid>>(ws_conns: &mut WsConnections, user_ws_id: Uuid, competition_ids: T){
    if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
        competition_ids.for_each(|cid| {
            println!("Adding subscription {}", cid); ws_user.subscriptions.competitions.insert(*cid);
        });
    };
}

pub async fn sub_to_teams(ws_conns: &mut WsConnections, user_ws_id: Uuid, toggle: bool){
    if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
        ws_user.subscriptions.teams = toggle
    };
}

pub async fn sub_to_players(ws_conns: &mut WsConnections, user_ws_id: Uuid, toggle: bool){
    if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
        ws_user.subscriptions.players = toggle
    };
}