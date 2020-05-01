use crate::models;
use crate::WsConnections;
use warp::ws;
use std::collections::HashMap;
use uuid::Uuid;

pub async fn publish_competitions(ws_conns: &mut WsConnections, competitions: &Vec<models::DbCompetition>){

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

pub async fn publish_series(ws_conns: &mut WsConnections, series: &Vec<models::DbSeries>){

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

pub async fn publish_matches(ws_conns: &mut WsConnections, matches: &Vec<models::DbMatch>, series_to_competitions: HashMap<Uuid, Uuid>){
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


pub async fn publish_teams(ws_conns: &mut WsConnections, teams: &Vec<models::DbTeam>){
    // TODO This doesnt include team-names that were mutated by their name-timestamp being 
    for (&uid, wsconn) in ws_conns.lock().await.iter_mut(){
        if wsconn.subscriptions.teams{
            match serde_json::to_string(&teams).as_ref(){
                Ok(subscribed_json) => {
                    if let Err(publish) = wsconn.tx.send(Ok(ws::Message::text(subscribed_json))){
                        println!("Error publishing update {:?} to {} : {}", &subscribed_json, uid, &publish)
                    };
                },
                Err(_) => println!("Error json serializing publisher update {:?} to {}", &teams, uid)
            };
        }
    };
}