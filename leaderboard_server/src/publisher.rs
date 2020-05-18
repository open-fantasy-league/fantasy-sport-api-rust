use crate::subscriptions::SubType;
use crate::types::leaderboards::*;
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;

impl Publishable<SubType> for Leaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::League => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.league_id))
                .collect(),
            SubType::Leaderboard => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.leaderboard_id))
                .collect(),
        }
    }
}

impl Publishable<SubType> for ApiLeaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard_detailed"
    }

    // Can commonise in a generic func between Leaderboard types
    // Would need to attach getters for league/leaderboard_id though
    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::League => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.league_id))
                .collect(),
            SubType::Leaderboard => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.leaderboard_id))
                .collect(),
        }
    }
}

impl Publishable<SubType> for Stat {
    fn message_type<'a>() -> &'a str {
        "stat"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::League => publishables
                .iter()
                .filter(|x| {
                    sub.ids
                        .contains(&id_map_opt.as_ref().unwrap().get(&x.leaderboard_id).unwrap())
                })
                .collect(),
            SubType::Leaderboard => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.leaderboard_id))
                .collect(),
        }
    }
}

impl Publishable<SubType> for ApiStat {
    fn message_type<'a>() -> &'a str {
        "stat"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::League => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.league_id))
                .collect(),
            SubType::Leaderboard => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.leaderboard_id))
                .collect(),
        }
    }
}
