#[macro_use]
extern crate diesel;
use dotenv::dotenv;
use std::env;
use warp::*;
use warp_ws_server::*;
use diesel_utils::{pg_pool, PgConn};
use uuid::Uuid;
mod handlers;
use handlers::*;
mod db;
mod schema;
mod models;
mod subscriptions;
mod publisher;
mod types;
mod drafting;
use drafting::generate_drafts;
use subscriptions::Subscriptions;
use async_trait::async_trait;
use futures::join;


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
            "sub_leagues" => sub_leagues(req, conn, ws_conns, user_ws_id).await,
            "sub_drafts" => sub_drafts(req, conn, ws_conns, user_ws_id).await,
            "sub_users" => sub_external_users(req, conn, ws_conns, user_ws_id).await,
            "insert_leagues" => insert_leagues(req, conn, ws_conns).await,
            "update_leagues" => update_leagues(req, conn, ws_conns).await,
            "insert_periods" => insert_periods(req, conn, ws_conns).await,
            "update_periods" => update_periods(req, conn, ws_conns).await,
            "insert_stat_multipliers" => insert_stat_multipliers(req, conn, ws_conns).await,
            "update_stat_multipliers" => update_stat_multipliers(req, conn, ws_conns).await,
            "insert_external_users" => insert_external_users(req, conn, ws_conns).await,
            "update_external_users" => update_external_users(req, conn, ws_conns).await,
            "insert_fantasy_teams" => insert_fantasy_teams(req, conn, ws_conns).await,
            "update_fantasy_teams" => update_fantasy_teams(req, conn, ws_conns).await,
            "insert_draft_queues" => insert_draft_queues(req, conn, ws_conns).await,
            "update_draft_queues" => update_draft_queues(req, conn, ws_conns).await,
            "insert_picks" => insert_picks(req, conn, ws_conns).await,
            "update_picks" => update_picks(req, conn, ws_conns).await,
            "update_draft_choices" => update_draft_choices(req, conn, ws_conns).await,
            uwotm8 => Err(Box::new(InvalidRequestError{description: uwotm8.to_string()}))
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("FANTASY_DB").expect("DATABASE_URL env var must be set");
    let pool = pg_pool(db_url);
    let draft_pgpool = pool.clone();
    let draft_handler = tokio::task::spawn(async move {
        drafting::draft_builder(draft_pgpool).await
    });

    let ws_conns =  warp_ws_server::ws_conns::<Subscriptions>();
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt)
        .map(move |ws: warp::ws::Ws, ws_conns|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn::<subscriptions::Subscriptions, A>(socket, pool, ws_conns))
        });
    //let server = warp::serve(ws_router).run(([127, 0, 0, 1], 3030));
    //draft_handler.await.map_err(|e|println!("{}", e.to_string()));
    join!(draft_handler, warp::serve(ws_router).run(([127, 0, 0, 1], 3030)));
}
