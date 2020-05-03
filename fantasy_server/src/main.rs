#[macro_use]
extern crate diesel;
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
    let pool = warp_ws_server::pg_pool();

   /* let pg_conn = warp::any().map(move || pool.clone()).and_then(|pool: PgPool| async move{ match pool.get(){
        Ok(conn) => Ok(conn),
        Err(_) => Err(reject::custom(PgPoolError)),
    }});*/
    /*let db_filter = warp::path::index().and(pg).and_then(|db: PooledPg| {
     futures::future::poll_fn(move || {
          let result = futures::try_ready!(tokio_threadpool::blocking(|| { /* do some stuff */ }));
          result.map(Async::Ready).map_err(internal_server_error)
     })
})
.and_then(|_| Ok("Set data in DB"));*/

    // .and(pg_conn).map(|conn: PgConn|{})
    let ws_conns =  warp_ws_server::ws_conns::<Subscriptions>();
    // Turn our "state" into a new Filter...
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());
    let mut methods: HashMap<String, warp_ws_server::WSMethod<Subscriptions>> = HashMap::new();
    methods.insert("upsert_leagues".to_string(), Box::new(upsert_leagues));
    let shareable_methods = Arc::new(methods);
    let methods_filt = warp::any().map(move || Arc::clone(&shareable_methods));
    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt).and(methods_filt)
        .map(move |ws: warp::ws::Ws, ws_conns, methods|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn::<Subscriptions>(socket, pool, ws_conns, methods))
        });
    warp::serve(ws_router).run(([127, 0, 0, 1], 3030)).await;
}
