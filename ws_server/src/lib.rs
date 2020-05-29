use uuid::Uuid;
use tokio::sync::{mpsc, Mutex};
use futures::{FutureExt, StreamExt};
use std::sync::Arc;
use std::collections::{HashMap};
use warp::ws;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
mod subscriptions;
pub use subscriptions::*;
mod publisher;
pub use publisher::*;
#[macro_use]
extern crate lazy_static;
use regex::Regex;



// There's so many different error handling libraries to choose from
// https://blog.yoshuawuyts.com/error-handling-survey/
// Eventually will probably use snafu
pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
// Arcs because warp needs to share WsConnections and WsMethods between all websocket connections (different threads)
// Maybe this lib should be agnostic to that, as it just focuses on a single connection
// However not sure how to "pull stuff out of Arcs", maybe by design that wouldnt work. And wouldnt be threadsafe.
pub type WSConnections<CustomSubType> = Arc<Mutex<HashMap<Uuid, WSConnection<CustomSubType>>>>;


// TODO make PgConn and Pgpool generic
// Really this library shouldnt give a shit about databases,
// Prob need to follow https://hoverbear.org/blog/optional-arguments/, rather than passing in an Option<PgConn> to some stuff
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
type PgPool = Pool<ConnectionManager<PgConnection>>;
type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

pub fn ws_conns<CustomSubType: std::cmp::Eq + std::hash::Hash>() -> WSConnections<CustomSubType>{
    Arc::new(Mutex::new(HashMap::new()))
}

pub struct WSConnection<CustomSubType: std::cmp::Eq + std::hash::Hash>{
    pub id: Uuid,
    pub subscriptions: Subscriptions<CustomSubType>,
    pub tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>,
}

impl<CustomSubType: std::cmp::Eq + std::hash::Hash> WSConnection<CustomSubType>{
    fn new<T: SubscriptionHandler<CustomSubType>>(tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>) -> WSConnection<CustomSubType> {
        WSConnection{id: Uuid::new_v4(), subscriptions: T::new(), tx: tx}
    }
}

#[derive(Serialize)]
pub struct WSMsgOut<'a, T: Serialize> {
    pub message_id: Uuid,
    pub mode: &'a str,
    pub message_type: &'a str,
    pub data: T
}

impl<'a, T: Serialize> WSMsgOut<'a, T>{
    pub fn resp(message_id: Uuid, message_type: &'a str, data: T) -> Self{
        return Self{message_id: message_id, message_type: message_type, mode: "resp", data: data}
    }

    pub fn push(message_type: &'a str, data: T) -> Self{
        return Self{message_id: Uuid::new_v4(), message_type: message_type, mode: "push", data: data}
    }

    pub fn error(data: T, message_id: Uuid) -> Self{
        return Self{message_id, message_type: "unknown", mode: "error", data}
    }
}

// Now Im using Enums properly, there's no leftover "default" in the pattern match to have to throw a custom error on
// (It will get a deserializing error if the "method" is invalid)

// #[derive(Debug, Clone)]
// pub struct InvalidRequestError{pub description: String}

// impl fmt::Display for InvalidRequestError{
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         write!(f, "Invalid request: {}", self.description)
//     }
// }

// impl std::error::Error for InvalidRequestError{
//     fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
//         // Generic error, underlying cause isn't tracked.
//         None
//     }
// }

pub fn ws_error_resp(error_msg: String, message_id: Uuid) -> ws::Message{
    ws::Message::text(
        serde_json::to_string(
            &WSMsgOut::error(error_msg, message_id)
        ).unwrap_or("Error serializing error message!".to_string())
    )
}

pub async fn handle_ws_conn<CustomSubType: std::cmp::Eq + std::hash::Hash, T: SubscriptionHandler<CustomSubType>, U: WSHandler<CustomSubType, CachesType>, CachesType: Clone>(
    ws: ws::WebSocket, pg_pool: PgPool, mut ws_conns: WSConnections<CustomSubType>, caches: CachesType
) {
    println!("handling ws conn");
    // https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs
    let (ws_send, mut ws_recv) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let ws_conn = WSConnection::new::<T>(tx);
    let ws_id = ws_conn.id;
    ws_conns.lock().await.insert(ws_conn.id, ws_conn);
    println!("post ws_conns lock");
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
            // but trying like this as worried about ownership of pool, so moving it into funcs
            Ok(msg) => match pg_pool.get(){
                // TODO for the caches im using, each msg just need to read-lock, not full lock. How do in rust?
                Ok(conn) => handle_ws_msg::<CustomSubType, U, CachesType>(
                    msg, conn, &mut ws_conns, ws_id, caches.clone()
                ).await,
                Err(e) => ws_error_resp(e.to_string(), Uuid::new_v4())
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

        // Feels unnecessary locking whole map just to get our tx (we moved it into the map, so cant access variable anymore)
        // Maybe something better
        if let Some(wsconn) = ws_conns.lock().await.get(&ws_id){
            if let Err(e) = wsconn.tx.send(resp){
                println!("Error sending regular msg to {}: {:?}", wsconn.id, e)
            };
        }
    }
}
#[derive(Serialize, Deserialize, Debug)]
struct Fudge{
    message_id: Uuid,
    data: serde_json::Value
}

async fn handle_ws_msg<CustomSubType: std::cmp::Eq + std::hash::Hash, U: WSHandler<CustomSubType, CachesType>, CachesType>(
    msg: ws::Message, conn: PgConn, ws_conns: &mut WSConnections<CustomSubType>, user_ws_id: Uuid, caches: CachesType
) -> ws::Message{
    dbg!(&msg);
    if msg.is_text(){
        match msg.to_str(){
            // Can't get await inside `and_then`/`map` function chains to work properly
            Ok(msg_str) => 
            {
                match U::ws_req_resp(msg_str.to_string(), conn, ws_conns, user_ws_id, caches).await{
                    Ok(text) => ws::Message::text(text),
                    Err(e) => {
                        dbg!(&e);
                        println!("{:?}", e.source());
                        // Might be better to return message-id attached to the error. but thats harder
                        lazy_static! {
                            static ref RE: Regex = Regex::new(r#".*?"message_id":"([^"]+)""#).unwrap();
                        }
                        let re_match = RE.find(msg_str).map(|x|x.as_str());
                        let message_id = match re_match{
                            Some(msg_id_str) => Uuid::parse_str(msg_id_str).unwrap(),
                            None => Uuid::new_v4()
                        };
                        ws_error_resp(e.to_string(), message_id)
                    }
                }
            }
            ,
            Err(_) => ws_error_resp(String::from("wtf. How does msg.to_str fail?"), Uuid::new_v4())
        }
    }
    else if msg.is_ping(){
        // currently no pong method? https://docs.rs/warp/0.2.2/warp/filters/ws/struct.Message.html#method.ping
        // should I add?
        ws::Message::text("{'mode': 'pong'}")
    }
    else{
        ws_error_resp(String::from("Unexpected message type received"), Uuid::new_v4())
    }
}

#[async_trait]
pub trait WSHandler<CustomSubType: std::cmp::Eq + std::hash::Hash, CachesType>{
    async fn ws_req_resp(
        msg: String, conn: PgConn, ws_conns: &mut WSConnections<CustomSubType>, user_ws_id: Uuid, caches: CachesType
    ) -> Result<String, BoxError>;
}


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
