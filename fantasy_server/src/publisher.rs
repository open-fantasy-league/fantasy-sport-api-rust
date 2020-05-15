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

impl Publishable<SubType> for Period {
    fn message_type<'a>() -> &'a str {
        "period"
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

impl Publishable<SubType> for StatMultiplier {
    fn message_type<'a>() -> &'a str {
        "stat_multiplier"
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

impl Publishable<SubType> for FantasyTeam {
    fn message_type<'a>() -> &'a str {
        "fantasy_team"
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
            SubType::Draft => warp_ws_server::this_should_never_happen(
                publishables,
                "FantasyTeam published for Draft",
            ),
            SubType::User => publishables
                .iter()
                .filter(|x| sub.ids.contains(&x.external_user_id))
                .collect(),
        }
    }
}

impl Publishable<SubType> for Pick {
    fn message_type<'a>() -> &'a str {
        "pick"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            // SubType::League => publishables
            //     .iter()
            //     .filter(|x| sub.ids.contains(&x.league_id))
            //     .collect(),
            SubType::Draft => warp_ws_server::this_should_never_happen(
                publishables,
                "FantasyTeam published for Draft",
            ),
            // SubType::User => publishables
            //     .iter()
            //     .filter(|x| sub.ids.contains(&x.external_user_id))
            //     .collect(),
            _ => panic!("fudge"),
        }
    }

    // fn subscription_id_map(
    //     conn: Option<&PgConn>,
    //     publishables: &Vec<Self>,
    // ) -> Result<HashMap<Uuid, Uuid>, BoxError> {
    //     let id_map = db::get_draft_ids_for_picks(
    //         conn.unwrap(),
    //         &publishables.iter().map(|p| p.pick_id).collect(),
    //     )?;
    //     Ok(id_map.into_iter().collect())
    // }
}

impl Publishable<SubType> for ActivePick {
    fn message_type<'a>() -> &'a str {
        "active_pick"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        _: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::League => warp_ws_server::this_should_never_happen(
                publishables,
                "ActivePick published for League",
            ),
            SubType::Draft => warp_ws_server::this_should_never_happen(
                publishables,
                "ActivePick published for Draft",
            ),
            SubType::User => warp_ws_server::this_should_never_happen(
                publishables,
                "ActivePick published for User",
            )
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
