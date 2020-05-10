use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use url;
use uuid::Uuid;
use crate::types::thisisshit::*;
use tokio::sync::Mutex;
use std::sync::Arc;
use warp_ws_server::BoxError;
// TODO am i using a different library to what warps using?
// i.e. can dependencies be reduced?


pub async fn listen_pick_results(result_port: u16, teams_and_players_mut: Arc<Mutex<Option<ApiTeamsAndPlayers>>>) -> Result<(), BoxError>{
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
                let mut state = teams_and_players_mut.lock().await;
                *state = Some(data);
            }
        }
        // if msg.is_text() || msg.is_binary() {
        //     ws_stream.send(msg).await?;
        // }
    }
    Ok(())
}