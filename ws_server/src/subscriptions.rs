use crate::{BoxError, PgConn};
use std::collections::{HashMap};
use uuid::Uuid;
// Not commonised yet
//Plans:
//subscriptions -> singular-subscription
// worth holding a reverse map?
// i.e. subscriptions -> users rather than users -> subscription
pub trait Subscriptions {
    fn new() -> Self;
}

pub trait Publishable {
    fn message_type<'a>() -> &'a str;
    fn subscription_map_key(&self) -> Uuid;
    fn subscription_id_map(conn: &PgConn, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError> where Self: Sized;
}