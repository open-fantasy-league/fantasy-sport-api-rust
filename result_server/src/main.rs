#[macro_use]
extern crate diesel;
extern crate dotenv;
//use std::collections::HashMap;
use warp::{self, Filter, get, post, path, body, reject};
mod db_pool;
use db_pool::{pg_pool, PgPool, PgConn};
use serde::{Deserialize, Serialize};
use serde_json::Value;
mod schema;
mod models;
use models::*;
mod db;
use db::*;

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

#[derive(Deserialize, Serialize)]
struct League {
    code: String,
    name: String,
    start: chrono::DateTime::<chrono::prelude::Utc>,
    end: chrono::DateTime::<chrono::prelude::Utc>,
    meta: Option<Value>
}

/*#[derive(Queryable, QueryableByName)]
struct Version {
    #[sql_type = "Text"]
    version: String
}*/

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

    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let league_results = warp::path!("league" / u32).map(|league_id| format!("League id {}", league_id));
    let series_results = warp::path!("series" / u64).map(|series_id| format!("Series id {}", series_id));
    //curl -X POST -H "Content-Type: application/json" -d '{"code": "chumpions_leageu", "name": "The champsions league 2020", "start": 0, "end": 22, "meta": {"extra": "info", "you": [2, 3, 4]}}' -v '127.0.0.1:3030/competition'
    let post_league = post()
        .and(path("competitions"))
        .and(body::json())
        .and(pg_conn)
        .map(|comp: NewCompetition, conn: PgConn|{
            /*let sql = "SELECT version();";
            let result = sql_query(sql)
    //.bind::<Text, _>("version()")
    .load::<Version>(&conn);
            //let result = sql_query(sql).get_results(&conn);
            league.meta = Some(json!(vec![(String::from("version"), result.unwrap()[0].version.clone())].into_iter().collect::<HashMap<_, _>>()));*/
            // Not just passing whole comp in as single arg, as for creating comps in tests and
            // stuff might be more akward
            let created = create_competition(&conn, &comp);//, &league.meta, &league.timespan);
            match created{
                Ok(c) => warp::reply::json(&c),
                Err(e) => warp::reply::json(&e.to_string())
            }
    });
    // https://github.com/seanmonstar/warp/blob/master/examples/body.rs
    let get_routes = get().and(league_results.or(series_results).or(hello));
    let post_routes = post_league;
    warp::serve(get_routes.or(post_routes)).run(([127, 0, 0, 1], 3030)).await;
}
