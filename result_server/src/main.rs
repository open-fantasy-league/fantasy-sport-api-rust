#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use] extern crate frunk;
use dotenv::dotenv;
use std::env;
use warp_ws_server;
use warp::*;
mod schema;
mod db;
mod types;
mod handlers;
use handlers::*;
mod publisher;
mod subscriptions;
use uuid::Uuid;
use warp_ws_server::*;
use async_trait::async_trait;


pub type WSConnections_ = warp_ws_server::WSConnections<subscriptions::Subscriptions>;
pub type WSConnection_ = warp_ws_server::WSConnection<subscriptions::Subscriptions>;


struct A{
}

#[async_trait]
impl WSHandler<subscriptions::Subscriptions> for A{

    async fn ws_req_resp(
        msg: String, conn: PgConn, ws_conns: &mut WSConnections<subscriptions::Subscriptions>, user_ws_id: Uuid
    ) -> Result<String, BoxError>{
        let req: WSReq = serde_json::from_str(&msg)?;
        println!("{}", &req.data);
        match req.method{
            // "sub_competitions" => sub_competitions(req, conn, ws_conns, user_ws_id).await,
            // "sub_teams" => sub_teams(req, conn, ws_conns, user_ws_id).await,
            // "insert_competitions" => insert_competitions(req, conn, ws_conns, user_ws_id).await,
            // "update_competitions" => update_competitions(req, conn, ws_conns, user_ws_id).await,
            // "upsert_series" => upsert_series(req, conn, ws_conns, user_ws_id).await,
            // "upsert_matches" => upsert_matches(req, conn, ws_conns, user_ws_id).await,
            // "upsert_series_teams" => upsert_series_teams(req, conn, ws_conns, user_ws_id).await,
            // "upsert_teams" => upsert_teams(req, conn, ws_conns, user_ws_id).await,
            // "upsert_players" => upsert_players(req, conn, ws_conns, user_ws_id).await,
            // "upsert_team_players" => upsert_team_players(req, conn, ws_conns, user_ws_id).await,
            // "upsert_team_match_results" => upsert_team_match_results(req, conn, ws_conns, user_ws_id).await,
            // "upsert_team_series_results" => upsert_team_series_results(req, conn, ws_conns, user_ws_id).await,
            // "upsert_player_match_results" => upsert_player_match_results(req, conn, ws_conns, user_ws_id).await,
            uwotm8 => Err(Box::new(InvalidRequestError{description: uwotm8.to_string()}))
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("RESULT_DB").expect("DATABASE_URL env var must be set");
    let pool = warp_ws_server::pg_pool(db_url);

    let ws_conns = warp_ws_server::ws_conns::<subscriptions::Subscriptions>();
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt)
        .map(move |ws: warp::ws::Ws, ws_conns|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn::<subscriptions::Subscriptions, A>(socket, pool, ws_conns))
        });
    warp::serve(ws_router).run(([127, 0, 0, 1], 3030)).await;
}
