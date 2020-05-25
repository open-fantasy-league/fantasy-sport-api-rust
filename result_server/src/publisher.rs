use crate::subscriptions::SubType;
use crate::types::{competitions::*, teams::*};
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;

impl Publishable<SubType> for Competition {
    fn message_type<'a>() -> &'a str {
        "competition_update"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.competition_id))
                .collect(),
            SubType::Team => warp_ws_server::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for ApiCompetition {
    fn message_type<'a>() -> &'a str {
        "competition"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.competition_id))
                .collect(),
            SubType::Team => warp_ws_server::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for ApiTeamWithPlayersHierarchy{
    fn message_type<'a>() -> &'a str {
        "team_and_players"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.team_id))
                    .collect()
            }
            SubType::Competition => warp_ws_server::this_should_never_happen(publishables, "ApiTeamWithPlayersHierarchy published for Comp")
        }
    }
}