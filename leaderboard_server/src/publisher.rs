use std::collections::HashMap;
use uuid::Uuid;
use warp_ws_server::*;
use diesel_utils::PgConn;
use crate::types::leaderboards::*;
use crate::subscriptions::SubType;
use crate::db;

impl Publishable<SubType> for Leaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard"
    }

    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, _: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>{
        Ok(
            match sub.all{
                // TODO anything nicer than iter->colelct?
                true => publishables.iter().collect(),
                false => {
                    match sub_type{
                        SubType::League => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.league_id)).collect()
                        },
                        SubType::Leaderboard => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.leaderboard_id)).collect()
                        }
                    }
                }
            }
        )
    }
}

impl Publishable<SubType> for ApiLeaderboard {
    fn message_type<'a>() -> &'a str {
        "leaderboard_detailed"
    }

    // Can commonise in a generic func between Leaderboard types
    // Would need to attach getters for league/leaderboard_id though
    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, _: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>{
        Ok(
            match sub.all{
                // TODO anything nicer than iter->colelct?
                true => publishables.iter().collect(),
                false => {
                    match sub_type{
                        SubType::League => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.league_id)).collect()
                        },
                        SubType::Leaderboard => {
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&x.leaderboard_id)).collect()
                        }
                    }
                }
            }
        )
    }
}

impl Publishable<SubType> for Stat {
    fn message_type<'a>() -> &'a str {
        "stat"
    }

    fn subscribed_publishables<'b>(publishables: &'b Vec<Self>, sub: &mut Subscription, sub_type: &SubType, conn: Option<&PgConn>) -> Result<Vec<&'b Self>, BoxError>{
        Ok(
            match sub.all{
                // TODO anything nicer than iter->colelct?
                true => publishables.iter().collect(),
                false => {
                    match sub_type{
                        SubType::League => {
                            let id_map: HashMap<Uuid, Uuid> = db::get_league_ids_to_leaderboard_ids(
                                conn.unwrap(), publishables.iter().map(|s| s.leaderboard_id).collect()
                            )?.into_iter().collect();
                            publishables.iter()
                            .filter(|x| sub.ids.contains(&id_map.get(&x.leaderboard_id).unwrap())).collect()
                    },
                    SubType::Leaderboard => publishables.iter()
                            .filter(|x| sub.ids.contains(&x.leaderboard_id)).collect()
                    }
                }
            }
        )
    }
}