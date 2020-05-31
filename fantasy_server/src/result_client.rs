use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use url;
use uuid::Uuid;
use crate::types::thisisshit::*;
use tokio::sync::{Mutex, MutexGuard};
use std::sync::Arc;
use warp_ws_server::BoxError;
use std::collections::HashMap;
use chrono::{Utc, DateTime};
use std::ops::RangeBounds;
// TODO am i using a different library to what warps using?
// i.e. can dependencies be reduced?


pub async fn listen_team_updates(
    result_addr: String, result_port: u16, player_position_cache_mut: Arc<Mutex<Option<HashMap<Uuid, String>>>>, player_team_cache_mut: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>
) -> Result<(), BoxError>{
    // connect to websocket on port
    // subscribe to teams/players.
    //build a map of players teams/positions
    // update map with new messages
    // use arc to share map with draft-thread
    let url = url::Url::parse(&format!("ws://{}:{}", result_addr, result_port)).unwrap();

    //let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();
    //tokio::spawn(read_stdin(stdin_tx));

    let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    //let (write, read) = ws_stream.split();
    let sub_teams_msg = format!("{{\"method\": \"SubTeam\", \"data\": {{\"toggle\": true}}, \"message_id\": \"{}\"}}", Uuid::new_v4());
    ws_stream.send(Message::text(sub_teams_msg)).await?;
    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        println!("Msg from result server: {:?}", msg);
        match serde_json::from_str(&msg.to_text()?)?{
            ResultMsgs::SubTeam{message_id: _, data, mode: _} => {
                let now = Utc::now();
                println!("pre lock acquire");
                let player_position_cache = player_position_cache_mut.lock().await;
                let player_team_cache = player_team_cache_mut.lock().await;
                println!("pre update_team_and_position_maps");
                update_team_and_position_maps(
                    &data, now, player_position_cache, player_team_cache
                );
                // *player_position_cache = Some(positions);
                // *player_team_cache = Some(teams);
                println!("Built player position and team maps")
            },
            // TODO can probably commonise this
            ResultMsgs::team_and_players{message_id: _, data, mode: _} => {
                let now = Utc::now();
                println!("pre lock acquire");
                let player_position_cache = player_position_cache_mut.lock().await;
                let player_team_cache = player_team_cache_mut.lock().await;
                println!("pre update_team_and_position_maps");
                update_team_and_position_maps(
                    &data, now, player_position_cache, player_team_cache
                );
                // *player_position_cache = Some(positions);
                // *player_team_cache = Some(teams);
                println!("Built player position and team maps")
            }
        }
        // if msg.is_text() || msg.is_binary() {
        //     ws_stream.send(msg).await?;
        // }
    }
    println!("Unexpectedly ended listen-pick-results loop");
    Ok(())
}

fn update_team_and_position_maps(
    teams_and_players: &Vec<ApiTeamWithPlayersHierarchy>,
    time: DateTime<Utc>,
    mut position_map_mut: MutexGuard<Option<HashMap<Uuid, String>>>, mut team_map_mut: MutexGuard<Option<HashMap<Uuid, Uuid>>>
){
    //TODO could probably do fancy and build as iterate. no inserts
    if let None = *position_map_mut{
        *position_map_mut = Some(HashMap::new());
    }
    if let None = *team_map_mut{
        *team_map_mut = Some(HashMap::new());
    }
    // We only care about the latest team/position
    // (That might not technically be true, i.e. if a future transfer is confirmed, next team might already be in db)
    teams_and_players.into_iter().for_each(|t|{
        let team_id = t.team_id;
        if let Some(players) = &t.players{
            players.iter().for_each(|tp|{
                if tp.timespan.contains(&time){
                    // could `map` rather than as_mut.unwrap
                    (*team_map_mut).as_mut().unwrap().insert(tp.player.player_id, team_id);
                };
                if let Some(positions) = &tp.player.positions{
                    positions.iter().for_each(|pos|{
                        if pos.timespan.contains(&time){
                            (*position_map_mut).as_mut().unwrap().insert(tp.player.player_id, pos.position.clone());
                        }
                    })
                }
            });
        }
    });
}