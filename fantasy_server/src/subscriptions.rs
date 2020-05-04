use uuid::Uuid;
use std::collections::HashSet;
pub struct Subscriptions{
    pub teams: bool,
    pub competitions: HashSet<Uuid>,
    pub all_competitions: bool
}

impl warp_ws_server::Subscriptions for Subscriptions{
    fn new() -> Subscriptions {
        Subscriptions{teams: false, competitions: HashSet::new(), all_competitions: false}
    }
}