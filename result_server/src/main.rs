
#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use] extern crate frunk;
use dotenv::dotenv;
use std::env;
use warp_ws_server;
use diesel_utils::{pg_pool, PgConn};
use warp::*;
mod schema;
mod db;
mod types;
mod handlers;
mod utils;
use handlers::*;
mod publisher;
mod subscriptions;
mod messages;
use messages::*;
use uuid::Uuid;
use warp_ws_server::*;
use async_trait::async_trait;


pub type WSConnections_ = warp_ws_server::WSConnections<subscriptions::SubType>;
pub type WSConnection_ = warp_ws_server::WSConnection<subscriptions::SubType>;


struct MyWsHandler{
}

#[async_trait]
impl WSHandler<subscriptions::SubType> for MyWsHandler{

    async fn ws_req_resp(
        msg: String, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid
    ) -> Result<String, BoxError>{
        let req: WSReq = serde_json::from_str(&msg)?;
        match req{
            WSReq::SubCompetition{message_id, data} => sub_competitions("SubCompetition", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::SubTeam{message_id, data} => sub_teams("SubTeam", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::Competition{message_id, data} => insert_competitions("Competition", message_id, data, conn, ws_conns).await,
            WSReq::CompetitionUpdate{message_id, data} => update_competitions("CompetitionUpdate", message_id, data, conn, ws_conns).await,
            WSReq::Series{message_id, data} => insert_series("Series", message_id, data, conn, ws_conns).await,
            WSReq::SeriesUpdate{message_id, data} => update_series("SeriesUpdate", message_id, data, conn, ws_conns).await,
            WSReq::Match{message_id, data} => insert_matches("Match", message_id, data, conn, ws_conns).await,
            WSReq::MatchUpdate{message_id, data} => update_matches("MatchUpdate", message_id, data, conn, ws_conns).await,
            WSReq::TeamSeriesResult{message_id, data} => insert_team_series_results("TeamSeriesResult", message_id, data, conn, ws_conns).await,
            WSReq::TeamSeriesResultUpdate{message_id, data} => update_team_series_results("TeamSeriesResultUpdate", message_id, data, conn, ws_conns).await,
            WSReq::TeamMatchResult{message_id, data} => insert_team_match_results("TeamMatchResult", message_id, data, conn, ws_conns).await,
            WSReq::TeamMatchResultUpdate{message_id, data} => update_team_match_results("TeamMatchResultUpdate", message_id, data, conn, ws_conns).await,
            WSReq::PlayerResult{message_id, data} => insert_player_results("PlayerResult", message_id, data, conn, ws_conns).await,
            WSReq::PlayerResultUpdate{message_id, data} => update_player_results("PlayerResultUpdate", message_id, data, conn, ws_conns).await,
            WSReq::Team{message_id, data} => insert_teams("Team", message_id, data, conn, ws_conns).await,
            WSReq::TeamUpdate{message_id, data} => update_teams("TeamUpdate", message_id, data, conn, ws_conns).await,
            WSReq::Player{message_id, data} => insert_players("Player", message_id, data, conn, ws_conns).await,
            WSReq::PlayerUpdate{message_id, data} => update_players("PlayerUpdate", message_id, data, conn, ws_conns).await,
            WSReq::TeamPlayer{message_id, data} => insert_team_players("TeamPlayer", message_id, data, conn, ws_conns).await,
            WSReq::TeamName{message_id, data} => insert_team_names("TeamName", message_id, data, conn, ws_conns).await,
            WSReq::PlayerName{message_id, data} => insert_player_names("PlayerName", message_id, data, conn, ws_conns).await,
            WSReq::PlayerPosition{message_id, data} => insert_player_positions("PlayerPosition", message_id, data, conn, ws_conns).await,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("RESULT_DB").expect("RESULT_DB env var must be set");
    let port = env::var("RESULT_PORT").expect("RESULT_PORT env var must be set").parse().expect("Port must be a number you lemming.");
    let pool = pg_pool(db_url);

    let ws_conns = warp_ws_server::ws_conns::<subscriptions::SubType>();
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt)
        .map(move |ws: warp::ws::Ws, ws_conns|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn::<subscriptions::SubType, subscriptions::MySubHandler, MyWsHandler>(
                socket, pool, ws_conns
            ))
        });
    warp::serve(ws_router).run(([127, 0, 0, 1], port)).await;
}
