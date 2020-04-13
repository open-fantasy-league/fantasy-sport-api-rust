#[macro_use]
extern crate diesel;
extern crate dotenv;
#[macro_use] // for the hlist macro
extern crate frunk;
extern crate frunk_core;
use std::collections::Bound;
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

#[derive(Deserialize, Serialize)]
struct Player {
    code: String,
    stats: Option<Value>
}


#[derive(Deserialize, Serialize)]
struct Team {
    code: String,
    players: Vec<Player>,
    result: Option<String>
}

// inpout series with all matches done
// input series with one match done, then later update (and maybe delete/patch that match)
// input series with no matches done, then later update
// avoid this by not having matches in series. have to POST request series, and then any matches
// separately. 
// They're joined/linked in db. but separate for CRUD shit

#[derive(Deserialize, Serialize)]
struct Series {
    league_id: u32,
    series_id: u64,
    result: Option<String>
}

#[derive(Deserialize, Serialize)]
struct Match {
    league_id: u32,
    series_id: u64,
    match_id: u64,
    teams: Vec<Team>,
    result: Option<String>
}

/*#[derive(Queryable, QueryableByName)]
struct Version {
    #[sql_type = "Text"]
    version: String
}*/

fn handle_ws_msg(msg: ws::Message) -> ws::Message{
    // No json-websocket response exists in warp yet
    match serde_json::to_string(&Series{league_id: 1, series_id: 2, result: None}){
        Ok(text) => ws::Message::text(text),
        Err(e) => {
            ws::Message::text(
                serde_json::to_string(
                    &ErrorResp{code: 500, message: e.to_string()}
                ).unwrap_or("Error serializing error message!".to_string())
            )
        }
    }
}


async fn handle_ws_conn(ws: ws::WebSocket){
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
            Ok(msg) => handle_ws_msg(msg),
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

    let pg_conn = warp::any().map(move || pool.clone()).and_then(|pool: PgPool| async move{ match pool.get(){
        Ok(conn) => Ok(conn),
        Err(_) => Err(reject::custom(PgPoolError)),
    }});
    /*let db_filter = warp::path::index().and(pg).and_then(|db: PooledPg| {
     futures::future::poll_fn(move || {
          let result = futures::try_ready!(tokio_threadpool::blocking(|| { /* do some stuff */ }));
          result.map(Async::Ready).map_err(internal_server_error)
     })
})
.and_then(|_| Ok("Set data in DB"));*/

    // .and(pg_conn).map(|conn: PgConn|{})

    let ws_router = warp::any().and(warp::ws()).map(|ws: warp::ws::Ws|{
            ws.on_upgrade(move |socket| handle_ws_conn(socket))
        });
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let league_results = warp::path!("league" / u32).map(|league_id| format!("League id {}", league_id));
    let series_results = warp::path!("series" / u64).map(|series_id| format!("Series id {}", series_id));
    //curl -X POST -H "Content-Type: application/json" -d '{"code": "chumpions_leageu", "name": "The champsions league 2020", "start": 0, "end": 22, "meta": {"extra": "info", "you": [2, 3, 4]}}' -v '127.0.0.1:3030/competition'
    // couldnt simplify the boilerplater of middle-two ands
    let post_competitions = post()
        .and(path("competitions"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewCompetition>, conn: PgConn| create_competitions(body, conn));
    let post_serieses = post()
        .and(path("series"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewSeries>, conn: PgConn| create_serieses(body, conn));
    let post_teams = post()
        .and(path("teams"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewTeam>, conn: PgConn| create_teams(body, conn));
    let post_matches = post()
        .and(path("matches"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<models::DbNewMatch>, conn: PgConn| create_matches(body, conn));
    let post_players = post()
        .and(path("players"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<ApiNewPlayer>, conn: PgConn| create_players(body, conn));
    let post_team_players = post()
        .and(path("team_players"))
        .and(body::json())
        .and(pg_conn.clone())
        .and_then(|body: Vec<models::DbNewTeamPlayer>, conn: PgConn| create_team_players(body, conn));
    let get_routes = get().and(league_results.or(series_results).or(hello));
    let post_routes = post_competitions.or(post_serieses).or(post_teams).or(post_matches)
        .or(post_players).or(post_team_players);
    warp::serve(ws_router.or(get_routes).or(post_routes)).run(([127, 0, 0, 1], 3030)).await;
}
