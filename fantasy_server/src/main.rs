#[macro_use]
extern crate diesel;
use dotenv::dotenv;
use std::env;
use warp::*;
use std::sync::Arc;
use warp_ws_server;
use std::collections::{HashSet, HashMap};
use uuid::Uuid;
mod handlers;
use handlers::*;
mod db;
mod schema;
mod models;

pub type WSConnections_ = warp_ws_server::WSConnections<Subscriptions>;

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

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("FANTASY_DB").expect("DATABASE_URL env var must be set");
    let pool = warp_ws_server::pg_pool(db_url);

    let ws_conns =  warp_ws_server::ws_conns::<Subscriptions>();
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let mut methods: HashMap<String, warp_ws_server::WSMethod<Subscriptions>> = HashMap::new();
    methods.insert("insert_leagues".to_string(), Box::new(insert_leagues));
    methods.insert("update_league".to_string(), Box::new(update_league));
    let shareable_methods = Arc::new(methods);
    let methods_filt = warp::any().map(move || Arc::clone(&shareable_methods));

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt).and(methods_filt)
        .map(move |ws: warp::ws::Ws, ws_conns, methods|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn::<Subscriptions>(socket, pool, ws_conns, methods))
        });
    warp::serve(ws_router).run(([127, 0, 0, 1], 3030)).await;
}
