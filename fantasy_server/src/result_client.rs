use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use url;
use uuid::Uuid;
use crate::types::thisisshit::*;
use tokio::sync::Mutex;
use std::sync::Arc;
use warp_ws_server::BoxError;
use std::collections::HashMap;
use chrono::{Utc, DateTime};
// TODO am i using a different library to what warps using?
// i.e. can dependencies be reduced?


pub async fn listen_pick_results(
    result_port: u16, player_position_cache_mut: Arc<Mutex<Option<HashMap<Uuid, &String>>>>, player_team_cache_mut: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>
) -> Result<(), BoxError>{
    // connect to websocket on port
    // subscribe to teams/players.
    //build a map of players teams/positions
    // update map with new messages
    // use arc to share map with draft-thread
    let url = url::Url::parse(&format!("ws://localhost:{}", result_port)).unwrap();

    //let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    //tokio::spawn(read_stdin(stdin_tx));

    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    //let (write, read) = ws_stream.split();
    let sub_teams_msg = format!("{{\"Method\": \"SubTeam\", \"data\": {{\"toggle\": true}}, \"message_id\": {}}}", Uuid::new_v4());
    ws_stream.send(Message::text(sub_teams_msg)).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        match serde_json::from_str(&msg.to_text()?)?{
            ResultMsgs::SubTeam{message_id, data, mode} => {
                let now = Utc::now();
                let (positions, teams) = build_team_and_position_maps(
                    &data, now
                );
                let mut player_position_cache = player_position_cache_mut.lock().await;
                let mut player_team_cache = player_team_cache_mut.lock().await;
                *player_position_cache = Some(positions);
                *player_team_cache = Some(teams);
                println!("Built player position and team maps")
            }
        }
        // if msg.is_text() || msg.is_binary() {
        //     ws_stream.send(msg).await?;
        // }
    }
    Ok(())
}

fn build_team_and_position_maps(teams_and_players: &ApiTeamsAndPlayers, time: DateTime<Utc>) -> (HashMap<Uuid, &String>, HashMap<Uuid, Uuid>){
    //TODO could probably do fancy and build as iterate. no inserts
    let mut positions: HashMap<Uuid, &String> = HashMap::new();
    let mut teams: HashMap<Uuid, Uuid> = HashMap::new();
    // We only care about the latest team/position
    // (That might not technically be true, i.e. if a future transfer is confirmed, next team might already be in db)
    teams_and_players.team_players.iter().for_each(|tp|{
        if tp.timespan.contains(&time){
            teams.insert(tp.player_id, tp.team_id);
        }
    });
    teams_and_players.players.iter().for_each(|player| {
        player.positions.iter().for_each(|p|{
            if p.timespan.contains(&time){
                positions.insert(player.player_id, &p.position);
            }
        })
    });
    (positions, teams)
}