use crate::{BoxError, PgConn};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
// Not commonised yet
//Plans:
//subscriptions -> singular-subscription
// worth holding a reverse map?
// i.e. subscriptions -> users rather than users -> subscription

pub struct Subscription {
    pub ids: HashSet<Uuid>,
    pub all: bool,
}

impl Subscription {
    fn new() -> Self {
        Self {
            ids: HashSet::new(),
            all: false,
        }
    }
}

pub struct Subscriptions<CustomSubType> {
    pub inner: HashMap<CustomSubType, Subscription>,
}

pub trait SubscriptionHandler<CustomSubType: std::cmp::Eq + std::hash::Hash> {
    fn new() -> Self;
    fn hmmm(&mut self) -> Subscriptions<CustomSubType>;
    fn get(&mut self, sub_type: &'static CustomSubType) -> &mut Subscription {
        self.hmmm().inner.get_mut(sub_type).unwrap()
    }
}

pub trait Publishable<CustomSubType> {
    fn message_type<'a>() -> &'a str;
    fn subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &CustomSubType,
        conn: Option<&PgConn>,
    ) -> Result<Vec<&'b Self>, BoxError>
    where
        Self: Sized;
}
