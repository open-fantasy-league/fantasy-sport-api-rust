#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use] extern crate frunk;

#[macro_use] extern crate frunk_core;
use dotenv::dotenv;
use std::env;
use warp_ws_server;
use std::collections::HashMap;
use futures::{FutureExt, StreamExt};
use warp::*;
use tokio::sync::Mutex;
use std::sync::Arc;
mod schema;
mod models;
mod db;
mod handlers;
use handlers::*;
mod publisher;
mod subscriptions;
use std::pin::Pin;
use uuid::Uuid;
use warp_ws_server::*;
pub type WSConnections_ = warp_ws_server::WSConnections<subscriptions::Subscriptions>;
pub type WSConnection_ = warp_ws_server::WSConnection<subscriptions::Subscriptions>;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("RESULT_DB").expect("DATABASE_URL env var must be set");
    let pool = warp_ws_server::pg_pool(db_url);
    let ws_conns = warp_ws_server::ws_conns::<subscriptions::Subscriptions>();

    let ws_conns =  warp_ws_server::ws_conns::<subscriptions::Subscriptions>();
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let mut methods: HashMap<String, warp_ws_server::WSMethod<subscriptions::Subscriptions>> = HashMap::new();
    methods.insert("upsert_competitions".to_string(), Box::new(upsert_competitions));
    //methods.insert("sub_teams".to_string(), Box::new(upsert_competitions));
    let shareable_methods = Arc::new(methods);
    let methods_filt = warp::any().map(move || Arc::clone(&shareable_methods));

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt).and(methods_filt)
        .map(move |ws: warp::ws::Ws, ws_conns, methods|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn(socket, pool, ws_conns, methods))
        });
    warp::serve(ws_router).run(([127, 0, 0, 1], 3030)).await;
}
