#[macro_use]
extern crate diesel;
mod db;
mod schema;
mod models;
mod subscriptions;
mod publisher;
mod types;
mod drafting;
mod messages;
mod result_client;
mod errors;
use dotenv::dotenv;
use std::env;
use warp::*;
use warp_ws_server::*;
use diesel_utils::{pg_pool, PgConn};
use uuid::Uuid;
mod handlers;
use handlers::*;
use messages::WSReq;
use async_trait::async_trait;
use futures::join;
use result_client::listen_pick_results;
use tokio::sync::Mutex;
use std::sync::Arc;
use std::collections::HashMap;


pub type WSConnections_ = warp_ws_server::WSConnections<subscriptions::SubType>;
pub type WSConnection_ = warp_ws_server::WSConnection<subscriptions::SubType>;

type Caches = (Arc<Mutex<Option<HashMap<Uuid, &String>>>>, Arc<Mutex<Option<HashMap<Uuid, Uuid>>>>);

struct MyWsHandler{
}

#[async_trait]
impl WSHandler<subscriptions::SubType, Caches> for MyWsHandler{

    async fn ws_req_resp(
        msg: String, conn: PgConn, ws_conns: &mut WSConnections_, user_ws_id: Uuid, caches: Caches
    ) -> Result<String, BoxError>{
        let req: WSReq = serde_json::from_str(&msg)?;
        match req{
            // For hardcoding method str, reflection in rust difficult
            WSReq::SubLeague{message_id, data} => sub_leagues("SubLeagues", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::SubDraft{message_id, data} => sub_drafts("SubDrafts", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::SubUser{message_id, data} => sub_external_users("SubUsers", message_id, data, conn, ws_conns, user_ws_id).await,
            WSReq::League{message_id, data} => insert_leagues("League", message_id, data, conn, ws_conns).await,
            WSReq::LeagueUpdate{message_id, data} => update_leagues("LeagueUpdate", message_id, data, conn, ws_conns).await,
            WSReq::Period{message_id, data} => insert_periods("Period", message_id, data, conn, ws_conns).await,
            WSReq::PeriodUpdate{message_id, data} => update_periods("PeriodUpdate", message_id, data, conn, ws_conns).await,
            WSReq::StatMultiplier{message_id, data} => insert_stat_multipliers("StatMultiplier", message_id, data, conn, ws_conns).await,
            WSReq::StatMultiplierUpdate{message_id, data} => update_stat_multipliers("StatMultiplierUpdate", message_id, data, conn, ws_conns).await,
            WSReq::ExternalUser{message_id, data} => insert_external_users("ExternalUser", message_id, data, conn, ws_conns).await,
            WSReq::ExternalUserUpdate{message_id, data} => update_external_users("ExternalUserUpdate", message_id, data, conn, ws_conns).await,
            WSReq::FantasyTeam{message_id, data} => insert_fantasy_teams("FantasyTeam", message_id, data, conn, ws_conns).await,
            WSReq::FantasyTeamUpdate{message_id, data} => update_fantasy_teams("FantasyTeam", message_id, data, conn, ws_conns).await,
            WSReq::DraftQueue{message_id, data} => insert_draft_queues("DraftQueue", message_id, data, conn, ws_conns).await,
            //WSReq::DraftQueueUpdate{message_id, data} => update_draft_queues("DraftQueueUpdate", message_id, data, conn, ws_conns).await,
            WSReq::Pick{message_id, data} => insert_picks("Pick", message_id, data, conn, ws_conns).await,
            WSReq::PickUpdate{message_id, data} => update_picks("PickUpdate", message_id, data, conn, ws_conns).await,
            WSReq::ActivePick{message_id, data} => upsert_active_picks(
                "ActivePick", message_id, data, conn, ws_conns, caches.0, caches.1
            ).await,
            WSReq::DraftChoiceUpdate{message_id, data} => update_draft_choices("DraftChoiceUpdate", message_id, data, conn, ws_conns).await,
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let db_url = env::var("FANTASY_DB").expect("DATABASE_URL env var must be set");
    let port: u16 = env::var("FANTASY_PORT").expect("FANTASY_PORT env var must be set").parse().expect("Port must be a number you lemming.");
    let result_port: u16 = env::var("Result_PORT").expect("RESULT_PORT env var must be set").parse().expect("Port must be a number you lemming.");

    //let teams_and_players_mut: Arc<Mutex<Option<ApiTeamsAndPlayers>>> = Arc::new(Mutex::new(None));
    let player_position_cache: Arc<Mutex<Option<HashMap<Uuid, &String>>>> = Arc::new(Mutex::new(None));
    let player_team_cache: Arc<Mutex<Option<HashMap<Uuid, Uuid>>>> = Arc::new(Mutex::new(None));
    let pool = pg_pool(db_url);
    let ws_conns =  warp_ws_server::ws_conns::<subscriptions::SubType>();
    // Is PgPool thread-safe? its not behind an arc...does it need to be?
    // maybe the clone is just make 3 seaprate pg-pool which is kind of fine.
    let draft_pgpool = pool.clone();
    let draft_handler_pool = pool.clone();

    let draft_builder_ws_conns = ws_conns.clone();
    let draft_handler_ws_conns = ws_conns.clone();
    let draft_builder = tokio::task::spawn(async move {
        drafting::draft_builder(draft_pgpool, draft_builder_ws_conns).await
    });
    // let draft_handler = tokio::task::spawn(async move {
    //     drafting::draft_builder(draft_pgpool).await
    // });

    let ws_conns_filt = warp::any().map(move || ws_conns.clone());

    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt)
        .map(move |ws: warp::ws::Ws, ws_conns|{
            let pool = pool.clone();
            let caches = (player_position_cache.clone(), player_team_cache.clone());
            ws.on_upgrade(move |socket| warp_ws_server::handle_ws_conn::<subscriptions::SubType, subscriptions::MySubHandler, MyWsHandler, Caches>(
                socket, pool, ws_conns, caches
            ))
        });
    //let server = warp::serve(ws_router).run(([127, 0, 0, 1], 3030));
    //draft_handler.await.map_err(|e|println!("{}", e.to_string()));
    join!(
        listen_pick_results(result_port, player_position_cache.clone(), player_team_cache.clone()),
        drafting::draft_handler(draft_handler_pool, player_position_cache.clone(), player_team_cache.clone(), draft_handler_ws_conns),
        draft_builder,
        warp::serve(ws_router).run(([127, 0, 0, 1], port)));
}
