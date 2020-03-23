use warp::{Filter, get, post, path, body};
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    start: u32,
    end: u32,
    meta: Option<Value>
}

#[tokio::main]
async fn main() {
    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(|name| format!("Hello, {}!", name));
    let league_results = warp::path!("league" / u32).map(|league_id| format!("League id {}", league_id));
    let series_results = warp::path!("series" / u64).map(|series_id| format!("Series id {}", series_id));
    //curl -X POST -H "Conation/json" -d '{"code": "chumpions_leageu", "name": "The champsions league 2020", "start": 0, "end": 22, "meta": {"fuck": "off", "you": [2, 3, 4]}}' -v '127.0.0.1:3030/league'
    let post_league = post()
        .and(path("league"))
        .and(body::json())
        .map(|league: League|{
            warp::reply::json(&league)
    });
    // https://github.com/seanmonstar/warp/blob/master/examples/body.rs
    let get_routes = get().and(league_results.or(series_results).or(hello));
    let post_routes = post_league;
    warp::serve(get_routes.or(post_routes)).run(([127, 0, 0, 1], 3030)).await;
}
