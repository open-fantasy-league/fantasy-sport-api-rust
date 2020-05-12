use crate::{WSConnection};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub struct Subscription {
    pub ids: HashSet<Uuid>,
    pub all: bool,
}

impl Subscription {
    pub fn new() -> Self {
        Self {
            ids: HashSet::new(),
            all: false,
        }
    }
}

pub type Subscriptions<CustomSubType> = HashMap<CustomSubType, Subscription>;

pub trait SubscriptionHandler<CustomSubType: std::cmp::Eq + std::hash::Hash> {
    fn new() -> Subscriptions<CustomSubType>;
}

//Just to avoid having to add .unwrap(), when in this scenario its 100% safe
pub trait GetEz<CustomSubType> {
    fn get_ez(&mut self, sub_type: &CustomSubType) -> &mut Subscription;
}

impl<CustomSubType: std::cmp::Eq + std::hash::Hash> GetEz<CustomSubType>
    for Subscriptions<CustomSubType>
{
    fn get_ez(&mut self, sub_type: &CustomSubType) -> &mut Subscription {
        self.get_mut(sub_type).unwrap()
    }
}

pub async fn sub<'a, T: Iterator<Item = &'a Uuid>, CustomSubType: std::cmp::Eq + std::hash::Hash>(
    sub_type: &CustomSubType, ws_user: &mut WSConnection<CustomSubType>, ids: T
){
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.get_mut(sub_type).unwrap().ids.insert(*id);
    });
}

pub async fn unsub<'a, T: Iterator<Item = &'a Uuid>, CustomSubType: std::cmp::Eq + std::hash::Hash>(
    sub_type: &CustomSubType, ws_user: &mut WSConnection<CustomSubType>, ids: T
){
    // TODO failure handling, does it panic?
    ids.for_each(|id| {
        println!("Adding subscription {}", id); ws_user.subscriptions.get_mut(&sub_type).unwrap().ids.remove(id);
    });
}

pub async fn sub_all<CustomSubType: std::cmp::Eq + std::hash::Hash>(sub_type: &CustomSubType, ws_user: &mut WSConnection<CustomSubType>, toggle: bool){
    ws_user.subscriptions.get_mut(&sub_type).unwrap().all = toggle;
}