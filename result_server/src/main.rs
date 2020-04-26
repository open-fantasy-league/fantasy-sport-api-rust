#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use] // for the hlist macro
extern crate frunk;
extern crate frunk_core;
use std::collections::{Bound, HashMap, HashSet};
use std::fmt;
use std::iter::FromIterator;
use itertools::Itertools;
use chrono::{DateTime, Utc};
use futures::{FutureExt, StreamExt};
use warp::*;
use tokio::sync::{mpsc, Mutex};
use std::sync::Arc;
mod db_pool;
use db_pool::{pg_pool, PgPool, PgConn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
mod schema;
mod models;
mod db;
mod handlers;
use handlers::*;
mod utils;

pub type DieselTimespan = (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>);

// There's so many different error handling libraries to choose from
// https://blog.yoshuawuyts.com/error-handling-survey/
// Eventually will probably use snafu
type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
type WsConnections = Arc<Mutex<HashMap<Uuid, WsConnection>>>;

pub fn generic_ws_error(error_msg: String) -> ws::Message{
    ws::Message::text(
        serde_json::to_string(
            &ErrorResp{code: 500, message: error_msg}
        ).unwrap_or("Error serializing error message!".to_string())
    )
}

struct Subscriptions{
    teams: bool,
    players: bool,
    competitions: HashSet<Uuid>
}

impl Subscriptions{
    fn new() -> Subscriptions {
        Subscriptions{teams: false, players: false, competitions: HashSet::new()}
    }
}
enum Subscription{
    CompetitionSubscription,
    String
}

struct WsConnection{
    id: Uuid,
    subscriptions: Subscriptions,
    tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>
}

impl WsConnection{
    fn new(tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>) -> WsConnection {
        WsConnection{id: Uuid::new_v4(), subscriptions: Subscriptions::new(), tx: tx}
    }
}

#[derive(Debug, Clone)]
struct InvalidRequestError{method: String}

impl fmt::Display for InvalidRequestError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid request_type {}", self.method)
    }
}

impl std::error::Error for InvalidRequestError{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

async fn sub_to_competitions<'a, T: Iterator<Item = &'a Uuid>>(ws_conns: &mut WsConnections, user_ws_id: Uuid, competition_ids: T){
    if let Some(ws_user) = ws_conns.lock().await.get_mut(&user_ws_id){
        competition_ids.for_each(|cid| {
            println!("Adding subscription {}", cid); ws_user.subscriptions.competitions.insert(*cid);
        });
    };
}

async fn publish_competitions(ws_conns: &mut WsConnections, competitions: &Vec<models::DbCompetition>){

    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed_comps: Vec<&models::DbCompetition>  = competitions.iter()
            .filter(|c| wsconn.subscriptions.competitions.contains(&c.competition_id)).collect();
        println!("subscribed_comps: {:?}", subscribed_comps);
        // TODO cache in-case lots of people have same filters
        let subscribed_comps_json_r = serde_json::to_string(&subscribed_comps);
        match subscribed_comps_json_r.as_ref(){
            Ok(subscribed_comps_json) => {
                if let Err(publish) = wsconn.tx.send(Ok(ws::Message::text(subscribed_comps_json))){
                    println!("Error publishing update {:?} to {} : {}", &subscribed_comps_json_r, uid, &publish)
                };
            },
            Err(_) => println!("Error json serializing publisher update {:?} to {}", &subscribed_comps_json_r, uid)
        };
    };
}

async fn publish_series(ws_conns: &mut WsConnections, series: &Vec<models::DbSeries>){

    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed: Vec<&models::DbSeries>  = series.iter()
            .filter(|s| wsconn.subscriptions.competitions.contains(&s.competition_id)).collect();
        println!("subscribed_series: {:?}", subscribed);
        // TODO cache in-case lots of people have same filters
        let subscribed_json_r = serde_json::to_string(&subscribed);
        match subscribed_json_r.as_ref(){
            Ok(subscribed_json) => {
                if let Err(publish) = wsconn.tx.send(Ok(ws::Message::text(subscribed_json))){
                    println!("Error publishing update {:?} to {} : {}", &subscribed_json, uid, &publish)
                };
            },
            Err(_) => println!("Error json serializing publisher update {:?} to {}", &subscribed_json_r, uid)
        };
    };
}

async fn publish_matches(ws_conns: &mut WsConnections, matches: &Vec<models::DbMatch>, series_to_competitions: HashMap<Uuid, Uuid>){
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        let subscribed: Vec<&models::DbMatch>  = matches.iter()
            .filter(|x| wsconn.subscriptions.competitions.contains(&series_to_competitions.get(&x.series_id).unwrap())).collect();
        println!("subscribed_series: {:?}", subscribed);
        // TODO cache in-case lots of people have same filters
        let subscribed_json_r = serde_json::to_string(&subscribed);
        match subscribed_json_r.as_ref(){
            Ok(subscribed_json) => {
                if let Err(publish) = wsconn.tx.send(Ok(ws::Message::text(subscribed_json))){
                    println!("Error publishing update {:?} to {} : {}", &subscribed_json, uid, &publish)
                };
            },
            Err(_) => println!("Error json serializing publisher update {:?} to {}", &subscribed_json_r, uid)
        };
    };
}

async fn ws_req_resp(msg: String, conn: PgConn, ws_conns: &mut WsConnections, user_ws_id: Uuid) -> Result<String, BoxError>{
    let req: WSReq = serde_json::from_str(&msg)?;
    println!("{}", &req.data);
    match req.method{
        "upsert_competitions" => {
            let dr = serde_json::from_value(req.data);
            let resp: Result<String, BoxError> = match dr{
                Ok(d) => {
                    println!("{:?}", &d);
                    let competitions_out_r= upsert_competitions(conn, d).await;
                    match competitions_out_r{
                        Ok(competitions_out) => {
                            // assume anything upserted the user wants to subscribe to
                            sub_to_competitions(ws_conns, user_ws_id, competitions_out.iter().map(|c| c.competition_id).collect::<HashSet<Uuid>>().iter()).await;
                            publish_competitions(ws_conns, &competitions_out).await;
                            println!("{:?}", &competitions_out);
                            serde_json::to_string(&competitions_out).map_err(|e| e.into())
                        },
                        Err(e) => Err(Box::new(e) as BoxError)
                    }
                },
                Err(e) => {Err(Box::new(e) as BoxError)}
            };
            resp
        },
        "upsert_serieses" => {
            let dr = serde_json::from_value(req.data);
            let resp: Result<String, BoxError> = match dr{
                Ok(d) => {
                    println!("{:?}", &d);
                    let series_out_r= upsert_serieses(conn, d).await;
                    match series_out_r{
                        Ok(series_out) => {
                            //let comp_ids: HashSet<Uuid> = series_out.iter().map(|s| s.competition_id).dedup().collect();
                            // assume anything upserted the user wants to subscribe to
                            // TODO check how turn map into iter
                            sub_to_competitions(ws_conns, user_ws_id, series_out.iter().map(|s| s.competition_id).collect::<HashSet<Uuid>>().iter()).await;
                            publish_series(ws_conns, &series_out).await;
                            serde_json::to_string(&series_out).map_err(|e| e.into())
                        },
                        Err(e) => Err(Box::new(e) as BoxError)
                    }
                },
                Err(e) => {Err(Box::new(e) as BoxError)}
            };
            resp
        },
        "upsert_matches" => {
            let dr = serde_json::from_value(req.data);
            let resp: Result<String, BoxError> = match dr{
                Ok(d) => {
                    println!("{:?}", &d);
                    let upserted_r= db::upsert_matches(&conn, d);//upsert_matches(&conn, d).await;
                    match upserted_r{
                        Ok(upserted) => {
                            let series_ids: Vec<Uuid> = upserted.iter().map(|s| s.series_id).dedup().collect();
                            let comp_and_series_ids_r = db::get_competition_ids_for_series(&conn, &series_ids);
                            match comp_and_series_ids_r{
                                Ok(comp_and_series_ids) => {
                                    // assume anything upserted the user wants to subscribe to
                                    sub_to_competitions(ws_conns, user_ws_id, comp_and_series_ids.iter().map(|x| &x.1)).await;
                                    publish_matches(ws_conns, &upserted, comp_and_series_ids.into_iter().collect()).await;
                                },
                                Err(e) => {
                                    println!("Error publishing upsert_serieses for comps: {}", e)
                                }
                            }
                            serde_json::to_string(&upserted).map_err(|e| e.into())
                        },
                        Err(e) => Err(Box::new(e) as BoxError)
                    }
                },
                Err(e) => {Err(Box::new(e) as BoxError)}
            };
            resp
        },
        // "upsert_matches" => {
        //     upsert_matches(conn, serde_json::from_value(req.data)?).await
        //     .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        // },
        // "upsert_teams" => {
        //     upsert_teams(conn, serde_json::from_value(req.data)?).await
        //     .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        // },
        // "upsert_players" => {
        //     upsert_players(conn, serde_json::from_value(req.data)?).await
        //     .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        // },
        // "upsert_team_players" => {
        //     upsert_team_players(conn, serde_json::from_value(req.data)?).await
        //     .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        // },
        //"upsert_series_teams" => upsert_series_teams(conn, serde_json::from_value(req.data)?),
        //"upsert_team_match_results" => upsert_team_match_results(conn, serde_json::from_value(req.data)?),
        //"upsert_player_match_results" => upsert_player_match_results(conn, serde_json::from_value(req.data)?),
        //"upsert_team_series_results" => upsert_team_series_results(conn, serde_json::from_value(req.data)?),
        uwotm8 => {
            // Think have to make it a string, to not piss-off borrow checker, as we are returning it from this func
            Err(Box::new(InvalidRequestError{method: uwotm8.to_string()}))
        }
    }
}

async fn handle_ws_msg(msg: ws::Message, conn: PgConn, ws_conns: &mut WsConnections, user_ws_id: Uuid) -> ws::Message{
    match msg.to_str(){
        Ok(msg_str) => match ws_req_resp(msg_str.to_string(), conn, ws_conns, user_ws_id).await{
            Ok(text) => ws::Message::text(text),
            Err(e) => generic_ws_error(e.to_string())
        },
        Err(e) => generic_ws_error(String::from("wtf. How does msg.to_str fail?"))
    }
}


async fn handle_ws_conn(ws: ws::WebSocket, pg_pool: PgPool, mut ws_conns: WsConnections) {
    // https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs
    let (ws_send, mut ws_recv) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let ws_conn = WsConnection::new(tx);
    let ws_id = ws_conn.id;
    // let ws_id = Uuid::new_v4();
    ws_conns.lock().await.insert(ws_conn.id, ws_conn);
    tokio::task::spawn(rx.forward(ws_send).map(|result| {
        if let Err(e) = result {
            eprintln!("websocket send error: {}", e);
        }
    }));

    while let Some(result) = ws_recv.next().await {
        // Wrapping in OK looks weird, but warps error handling is a bit....hmmmm
        // and this does kind of make sense to a user. you just get a ws msg through
        // you dont get a success/failure like http
        // https://github.com/seanmonstar/warp/issues/388
        let resp = Ok(match result {
            // Err handling looks a bit clunky, but want to only break on websocket error
            // (i.e. not pgpool error)
            // pgpool get should probably be deferred until after we unwrap/get websocket message
            // but trying like this as worried about ownership of pool, moving it into funcs
            Ok(msg) => match pg_pool.get(){
                Ok(conn) => handle_ws_msg(msg, conn, &mut ws_conns, ws_id).await,
                Err(e) => generic_ws_error(e.to_string())
            },
            Err(e) => {
                eprintln!("websocket error(uid=): {}", e);
                // If the websocket recv is broken, is it viable to try and send back through there was
                // an error? (Don't send actual error, maybe sensitive info? Who knows?
                if let Some(wsconn) = ws_conns.lock().await.get(&ws_id){
                    if let Err(e) = wsconn.tx.send(Ok(ws::Message::text("Unexpected recv error"))){
                        println!("Error sending Unexpected recv error msg to {}: {:?}", wsconn.id, e)
                    };
                }
                ws_conns.lock().await.remove(&ws_id);
                break;
            }
        });
        //let new_msg = format!("<User#>: {}", msg.to_str().unwrap_or("Well fuck"));
        //tx.send(Ok(ws::Message::text(new_msg.clone())));
        // Feels unnecessary locking whole map just to get our tx (we moved it into the map, so cant access variable anymore)
        // Maybe something better
        if let Some(wsconn) = ws_conns.lock().await.get(&ws_id){
            if let Err(e) = wsconn.tx.send(resp){
                println!("Error sending regular msg to {}: {:?}", wsconn.id, e)
            };
        }
    }
}

#[derive(Debug)]
struct PgPoolError;
impl reject::Reject for PgPoolError {}

#[tokio::main]
async fn main() {
    let pool = pg_pool();

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
    let ws_conns = Arc::new(Mutex::new(HashMap::new()));
    // Turn our "state" into a new Filter...
    let ws_conns_filt = warp::any().map(move || ws_conns.clone());
    let ws_router = warp::any().and(warp::ws()).and(ws_conns_filt).map(move |ws: warp::ws::Ws, ws_conns|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| handle_ws_conn(socket, pool, ws_conns))
        });
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let league_results = warp::path!("league" / u32).map(|league_id| format!("League id {}", league_id));
    let series_results = warp::path!("series" / u64).map(|series_id| format!("Series id {}", series_id));
    //curl -X POST -H "Content-Type: application/json" -d '{"code": "chumpions_leageu", "name": "The champsions league 2020", "start": 0, "end": 22, "meta": {"extra": "info", "you": [2, 3, 4]}}' -v '127.0.0.1:3030/competition'
    // couldnt simplify the boilerplater of middle-two ands
    /*let post_competitions = post()
        .and(path("competitions"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewCompetition>, conn: PgConn| upsert_competitions(conn, body));
    let post_serieses = post()
        .and(path("series"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewSeries>, conn: PgConn| upsert_serieses(conn, body));
    let post_teams = post()
        .and(path("teams"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewTeam>, conn: PgConn| upsert_teams(conn, body));
    let post_matches = post()
        .and(path("matches"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<models::DbNewMatch>, conn: PgConn| upsert_matches(conn, body));
    let post_players = post()
        .and(path("players"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewPlayer>, conn: PgConn| upsert_players(conn, body));
    let post_team_players = post()
        .and(path("team_players"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<models::DbNewTeamPlayer>, conn: PgConn| upsert_team_players(conn, body));
    let get_routes = get().and(league_results.or(series_results).or(hello));
    let post_routes = post_competitions.or(post_serieses).or(post_teams).or(post_matches)
        .or(post_players).or(post_team_players);*/
    //warp::serve(ws_router.or(get_routes).or(post_routes)).run(([127, 0, 0, 1], 3030)).await;
    warp::serve(ws_router).run(([127, 0, 0, 1], 3030)).await;
}
