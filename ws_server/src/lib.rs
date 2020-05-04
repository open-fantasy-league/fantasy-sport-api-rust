use uuid::Uuid;
use tokio::sync::{mpsc, Mutex};
use futures::{FutureExt, StreamExt, Future};
use std::sync::Arc;
use std::collections::{HashMap};
use std::fmt;
use std::pin::Pin;
use warp::ws;
mod db_pool;
pub use db_pool::{PgPool, PgConn, pg_pool};
pub mod utils;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
// There's so many different error handling libraries to choose from
// https://blog.yoshuawuyts.com/error-handling-survey/
// Eventually will probably use snafu
pub type BoxError = Box<dyn std::error::Error + Sync + Send + 'static>;
// Arcs because warp needs to share WsConnections and WsMethods between all websocket connections (different threads)
// Maybe this lib should be agnostic to that, as it just focuses on a single connection
// However not sure how to "pull stuff out of Arcs", maybe by design that wouldnt work. And wouldnt be threadsafe.
pub type WSConnections<T> = Arc<Mutex<HashMap<Uuid, WSConnection<T>>>>;
pub type WSMethod<T> = Box<dyn (Fn(WSReq, PgConn, &mut WSConnections<T>, Uuid) -> Pin<Box<dyn Future<Output=Result<String, BoxError>> + Send + Sync >>) + Send + Sync>;
//pub type WSMethod<T> = Box<dyn Fn(WSReq, PgConn, &mut WSConnections<T>, Uuid) -> Result<String, BoxError> + Send + Sync>;
// TODO this prob could be &str, but harder to get lifetimes to work
pub type WSMethods<T> = Arc<HashMap<String, WSMethod<T>>>;
pub trait Subscriptions {
    fn new() -> Self;
}

pub fn ws_conns<T: Subscriptions>() -> WSConnections<T>{
    Arc::new(Mutex::new(HashMap::new()))
}

pub struct WSConnection<T: Subscriptions>{
    pub id: Uuid,
    pub subscriptions: T,
    pub tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>
}

impl<T: Subscriptions> WSConnection<T>{
    fn new(tx: mpsc::UnboundedSender<Result<ws::Message, warp::Error>>) -> WSConnection<T> {
        WSConnection{id: Uuid::new_v4(), subscriptions: T::new(), tx: tx}
    }
}

#[derive(Serialize)]
pub struct WSMsgOut<T: Serialize> {
    pub message_id: Uuid,
    pub mode: String,
    pub message_type: String,
    pub data: T
}

impl<T: Serialize> WSMsgOut<T>{
    pub fn resp(message_id: Uuid, message_type: String, data: T) -> Self{
        return Self{message_id: message_id, message_type: message_type, mode: "resp".to_string(), data: data}
    }

    pub fn push(message_type: String, data: T) -> Self{
        return Self{message_id: Uuid::new_v4(), message_type: message_type, mode: "push".to_string(), data: data}
    }

    pub fn error(data: T) -> Self{
        return Self{message_id: Uuid::new_v4(), message_type: "unknown".to_string(), mode: "error".to_string(), data: data}
    }
}


#[derive(Deserialize)]
pub struct WSReq {
    pub message_id: Uuid,
    pub method: String,
    pub data: serde_json::Value
}

#[derive(Debug, Clone)]
pub struct InvalidRequestError{pub description: String}

impl fmt::Display for InvalidRequestError{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Invalid request: {}", self.description)
    }
}

impl std::error::Error for InvalidRequestError{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

pub fn ws_error_resp(error_msg: String) -> ws::Message{
    ws::Message::text(
        serde_json::to_string(
            &WSMsgOut::error(error_msg)
        ).unwrap_or("Error serializing error message!".to_string())
    )
}

pub async fn handle_ws_conn<T: Subscriptions, U: WSHandler<T>>(ws: ws::WebSocket, pg_pool: PgPool, mut ws_conns: WSConnections<T>) {
    // https://github.com/seanmonstar/warp/blob/master/examples/websockets_chat.rs
    let (ws_send, mut ws_recv) = ws.split();
    let (tx, rx) = mpsc::unbounded_channel();
    let ws_conn = WSConnection::new(tx);
    let ws_id = ws_conn.id;
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
                Ok(conn) => handle_ws_msg::<T, U>(msg, conn, &mut ws_conns, ws_id).await,
                Err(e) => ws_error_resp(e.to_string())
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

async fn handle_ws_msg<T: Subscriptions, U: WSHandler<T>>(
    msg: ws::Message, conn: PgConn, ws_conns: &mut WSConnections<T>, user_ws_id: Uuid
) -> ws::Message{
    match msg.to_str(){
        // Can't get await inside `and_then`/`map` function chains to work properly
        Ok(msg_str) => match U::ws_req_resp(msg_str.to_string(), conn, ws_conns, user_ws_id).await{
            Ok(text) => ws::Message::text(text),
            Err(e) => ws_error_resp(e.to_string())
        },
        Err(_) => ws_error_resp(String::from("wtf. How does msg.to_str fail?"))
    }
}

#[async_trait]
pub trait WSHandler<T: Subscriptions>{
    async fn ws_req_resp(
        msg: String, conn: PgConn, ws_conns: &mut WSConnections<T>, user_ws_id: Uuid
    ) -> Result<String, BoxError>;
}

// async fn ws_req_resp<T: Subscriptions>(
//     msg: String, conn: PgConn, ws_conns: &mut WSConnections<T>, user_ws_id: Uuid, methods: &WSMethods<T>
// ) -> Result<String, BoxError>{
//     let req: WSReq = serde_json::from_str(&msg)?;
//     println!("{}", &req.data);
//     let method = methods.get(&req.method.to_string())
//         .ok_or(Box::new(InvalidRequestError{description: req.method.to_string()}))?;
//     method(req, conn, ws_conns, user_ws_id).await
// }

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
