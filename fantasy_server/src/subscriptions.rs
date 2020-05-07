use uuid::Uuid;
use std::collections::HashSet;
pub struct Subscriptions{
    pub external_users: bool,
    pub leagues: HashSet<Uuid>,
    pub drafts: HashSet<Uuid>,
    pub all_leagues: bool,
    pub all_drafts: bool
}

impl warp_ws_server::Subscriptions for Subscriptions{
    fn new() -> Subscriptions {
        Subscriptions{external_users: false, leagues: HashSet::new(), drafts: HashSet::new(), all_leagues: false, all_drafts: false}
    }
}