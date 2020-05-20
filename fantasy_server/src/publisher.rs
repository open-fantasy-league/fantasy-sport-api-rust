use crate::subscriptions::SubType;
use crate::types::{drafts::*, fantasy_teams::*, leagues::*, users::*};
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;

impl Publishable<SubType> for League {
    fn message_type<'a>() -> &'a str {
        "league"
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
            SubType::Draft => {
                warp_ws_server::this_should_never_happen(publishables, "League published for Draft")
            }
            SubType::User => {
                warp_ws_server::this_should_never_happen(publishables, "League published for User")
            }
        }
    }
}

impl Publishable<SubType> for ApiLeague {
    fn message_type<'a>() -> &'a str {
        "league"
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
            SubType::Draft => {
                warp_ws_server::this_should_never_happen(publishables, "League published for Draft")
            }
            SubType::User => {
                warp_ws_server::this_should_never_happen(publishables, "League published for User")
            }
        }
    }
}

impl Publishable<SubType> for ApiDraft {
    fn message_type<'a>() -> &'a str {
        "draft"
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
            SubType::Draft => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.draft_id))
                .collect(),
            SubType::User => {
                warp_ws_server::this_should_never_happen(publishables, "Draft published for User")
            }
        }
    }
}


impl Publishable<SubType> for ExternalUser {
    fn message_type<'a>() -> &'a str {
        "user"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::League => {
                warp_ws_server::this_should_never_happen(publishables, "User published for League")
            }
            SubType::Draft => {
                warp_ws_server::this_should_never_happen(publishables, "League published for Draft")
            }
            SubType::User => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.external_user_id))
                .collect(),
        }
    }
}
