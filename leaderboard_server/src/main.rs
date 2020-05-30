

//https://github.com/emk/rust-musl-builder#making-diesel-work
extern crate openssl;
#[macro_use]
extern crate diesel;
extern crate dotenv;
use dotenv::dotenv;
use std::env;
use warp_ws_server;
use diesel_utils::{pg_pool, PgConn};
use warp::*;
mod schema;
mod db;
mod types;
mod handlers;
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

type Caches = ();

struct MyWsHandler{
}

#[async_trait]
impl WSHandler<subscriptions::SubType, Caches> for MyWsHandler{

    async fn ws_req_resp(
        msg: String, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid, _: Caches
    ) -> Result<String, BoxError>{
        let req: WSReq = serde_json::from_str(&msg)?;
        match req{
            WSReq::SubLeaderboard{message_id, data} => sub_leaderboards("SubLeaderboard", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::SubLeague{message_id, data} => sub_leagues("SubLeague", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::Leaderboard{message_id, data} => insert_leaderboards("Leaderboard", message_id, data, conn, ws_conns).await,
            WSReq::LeaderboardUpdate{message_id, data} => update_leaderboards("LeaderboardUpdate", message_id, data, conn, ws_conns).await,
            WSReq::LeaderboardGet{message_id, data} => get_latest_leaderboards("LeaderboardGet", message_id, data, conn, ws_conns).await,
            WSReq::Stat{message_id, data} => insert_stats("Stat", message_id, data, conn, ws_conns).await,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    println!("Doing stuff");
    let db_url = env::var("LEADERBOARD_DB").expect("LEADERBOARD_URL env var must be set");
    let port = env::var("LEADERBOARD_PORT").expect("LEADERBOARD_PORT env var must be set").parse().expect("Port must be a number you lemming.");
    let pool = pg_pool(db_url);

    let ws_conns = warp_ws_server::ws_conns::<subscriptions::SubType>();
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt)
        .map(move |ws: warp::ws::Ws, ws_conns|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| 
                warp_ws_server::handle_ws_conn::<subscriptions::SubType, subscriptions::MySubHandler, MyWsHandler, Caches>(
                    socket, pool, ws_conns, ()
                ))
        });
    warp::serve(ws_router).run(([0, 0, 0, 0], port)).await;
}