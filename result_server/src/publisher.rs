use crate::db;
use crate::subscriptions::SubType;
use crate::types::{competitions::*, series::*, teams::*, matches::*, results::*, players::*};
use diesel_utils::PgConn;
use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;
use crate::utils;

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
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
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
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for ApiSeriesNew {
    fn message_type<'a>() -> &'a str {
        "series"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.competition_id))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for Series {
    fn message_type<'a>() -> &'a str {
        "series_update"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.series_id).unwrap()))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for Match {
    fn message_type<'a>() -> &'a str {
        "matches_update"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                // let id_map: HashMap<Uuid, Uuid> = db::get_league_ids_to_leaderboard_ids(
                //     conn.unwrap(),
                //     publishables.iter().map(|s| s.leaderboard_id).collect(),
                // )?
                // .into_iter()
                // .collect();
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.series_id).unwrap()))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for ApiMatchNew {
    fn message_type<'a>() -> &'a str {
        "matches"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.series_id).unwrap()))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for TeamMatchResult {
    fn message_type<'a>() -> &'a str {
        "team_match_results"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.match_id).unwrap()))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}


impl Publishable<SubType> for PlayerResult {
    fn message_type<'a>() -> &'a str {
        "player_match_results"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.match_id).unwrap()))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for TeamSeriesResult {
    fn message_type<'a>() -> &'a str {
        "team_series_results"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Competition => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.series_id).unwrap()))
                    .collect()
            }
            SubType::Team => utils::this_should_never_happen(publishables, "Comp published for Team")
        }
    }
}

impl Publishable<SubType> for ApiTeam {
    fn message_type<'a>() -> &'a str {
        "team"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.team_id))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for TeamPlayer {
    fn message_type<'a>() -> &'a str {
        "team_player"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.team_id))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for TeamUpdate {
    fn message_type<'a>() -> &'a str {
        "team_update"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.team_id))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

// TODO should this come under team message?
impl Publishable<SubType> for TeamName {
    fn message_type<'a>() -> &'a str {
        "team_name"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.team_id))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

// TODO should this come under team message?
impl Publishable<SubType> for Team {
    fn message_type<'a>() -> &'a str {
        "team"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&x.team_id))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for ApiPlayer {
    fn message_type<'a>() -> &'a str {
        "player"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.player_id).unwrap()))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for Player {
    fn message_type<'a>() -> &'a str {
        "player"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().as_ref().unwrap().get(&x.player_id).unwrap()))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for PlayerUpdate {
    fn message_type<'a>() -> &'a str {
        "player_update"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.player_id).unwrap()))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for PlayerName {
    fn message_type<'a>() -> &'a str {
        "player_name"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.player_id).unwrap()))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}

impl Publishable<SubType> for PlayerPosition {
    fn message_type<'a>() -> &'a str {
        "player_position"
    }

    fn partial_subscribed_publishables<'b>(
        publishables: &'b Vec<Self>,
        sub: &mut Subscription,
        sub_type: &SubType,
        id_map_opt: &Option<HashMap<Uuid, Uuid>>,
    ) -> Vec<&'b Self> {
        match sub_type {
            SubType::Team => {
                publishables
                    .iter()
                    .filter(|x| sub.ids.contains(&id_map_opt.as_ref().unwrap().get(&x.player_id).unwrap()))
                    .collect()
            }
            SubType::Competition => utils::this_should_never_happen(publishables, "Team published for Comp")
        }
    }
}