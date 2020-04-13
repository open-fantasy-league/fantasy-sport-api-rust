#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use] // for the hlist macro
extern crate frunk;
extern crate frunk_core;
use std::collections::Bound;
use std::fmt;
use chrono::{DateTime, Utc};
use futures::{FutureExt, StreamExt};
use warp::*;
use tokio::sync::{mpsc};
mod db_pool;
use db_pool::{pg_pool, PgPool, PgConn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod schema;
mod models;
mod db;
mod handlers;
use handlers::*;
mod utils;

pub type DieselTimespan = (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>);

pub fn generic_ws_error(error_msg: String) -> ws::Message{
    ws::Message::text(
        serde_json::to_string(
            &ErrorResp{code: 500, message: error_msg}
        ).unwrap_or("Error serializing error message!".to_string())
    )
}

#[derive(Debug, Clone)]
struct InvalidRequestError<'a>{req_type: &'a str}

impl fmt::Display for InvalidRequestError<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid request_type {}", self.req_type)
    }
}

impl std::error::Error for InvalidRequestError<'_> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

async fn ws_req_resp(msg: String, conn: PgConn) -> Result<String, Box<dyn std::error::Error>>{
    let req: WSReq = serde_json::from_str(&msg)?;

    match req.request_type{
        "upsert_competitions" => {
            let dr = serde_json::from_value(req.data);
            match dr{
            Ok(d) => match upsert_competitions(conn, d).await{
                Ok(x) => match serde_json::to_string(&x){
                        Ok(fuck) => Ok(fuck),
                        Err(off) => Err(Box::new(off))
                    }
                Err(e) => Err(Box::new(e))
                //.and_then(|x| serde_json::to_string(&x)).map_err(Box::new)},
                }
            Err(e) => Err(Box::new(e))
            }
        },
        /*"upsert_serieses" => {
            upsert_serieses(conn, serde_json::from_value(req.data)?).await
            .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        },
        "upsert_matches" => {
            upsert_matches(conn, serde_json::from_value(req.data)?).await
            .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        },
        "upsert_teams" => {
            upsert_teams(conn, serde_json::from_value(req.data)?).await
            .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        },
        "upsert_players" => {
            upsert_players(conn, serde_json::from_value(req.data)?).await
            .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        },
        "upsert_team_players" => {
            upsert_team_players(conn, serde_json::from_value(req.data)?).await
            .and_then(|x| serde_json::to_string(&x)?).map_err(Box::new)
        },*/
        //"upsert_series_teams" => upsert_series_teams(conn, serde_json::from_value(req.data)?),
        //"upsert_team_match_results" => upsert_team_match_results(conn, serde_json::from_value(req.data)?),
        //"upsert_player_match_results" => upsert_player_match_results(conn, serde_json::from_value(req.data)?),
        //"upsert_team_series_results" => upsert_team_series_results(conn, serde_json::from_value(req.data)?),
        uwotm8 => {
            // fighting the borrow-checker here.
            // cos the error lives beyond this func, so implicit lifetimes arent enough
            //let gorge_cloney = &uwotm8.to_string().clone();
            Err(Box::new(InvalidRequestError{req_type: ""}))
        }
    }
    // No json-websocket response exists in warp yet
    //serde_json::to_string(&vec![1, 2, 34])
}

async fn handle_ws_msg(msg: ws::Message, conn: PgConn) -> ws::Message{
    match ws_req_resp(msg.to_str().unwrap().to_string(), conn).await{
        Ok(text) => ws::Message::text(text),
        Err(e) => generic_ws_error(e.to_string())
    }
}


async fn handle_ws_conn(ws: ws::WebSocket, pg_pool: PgPool){
    // https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs
    let (ws_send, mut ws_recv) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
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
                Ok(conn) => handle_ws_msg(msg, conn).await,
                Err(e) => generic_ws_error(e.to_string())
            },
            Err(e) => {
                eprintln!("websocket error(uid=): {}", e);
                // If the websocket recv is broken, is it viable to try and send back through there was
                // an error? (Don't send actual error, maybe sensitive info? Who knows?
                tx.send(Ok(ws::Message::text("Unexpected recv error")));
                break;
            }
        });
        //let new_msg = format!("<User#>: {}", msg.to_str().unwrap_or("Well fuck"));
        //tx.send(Ok(ws::Message::text(new_msg.clone())));
        tx.send(resp);
        //user_message(my_id, msg, &users).await;
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

    let ws_router = warp::any().and(warp::ws()).map(move |ws: warp::ws::Ws|{
            let pool = pool.clone();
            ws.on_upgrade(move |socket| handle_ws_conn(socket, pool))
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
