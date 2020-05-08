use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::publisher::Publishable;
use diesel_utils::{PgConn,DieselTimespan};
use std::collections::HashMap;
use crate::db;
use warp_ws_server::BoxError;

//https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(fantasy_team_id)]
pub struct FantasyTeam {
    pub fantasy_team_id: Uuid,
    pub name: String,
    pub league_id: Uuid,
    pub external_user_id: Uuid,
    pub meta: serde_json::Value,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "fantasy_teams"]
#[primary_key(fantasy_team_id)]
pub struct FantasyTeamUpdate {
    pub fantasy_team_id: Uuid,
    pub name: Option<String>,
    //pub league_id: Option<Uuid>,
    pub external_user_id: Option<Uuid>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(pick_id)]
pub struct Pick {
    pub pick_id: Uuid,
    pub fantasy_team_id: Uuid,
    pub player_id: Uuid,
    pub timespan: DieselTimespan,
    pub active: bool,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[primary_key(pick_id)]
#[table_name = "picks"]
pub struct PickUpdate {
    pub pick_id: Uuid,
    // These shouldnt be mutable
    // pub fantasy_team_id: Uuid,
    // pub player_id: Uuid,
    // pub timespan: DieselTimespan,
    pub active: Option<bool>,
}


impl Publishable for FantasyTeam {

    fn message_type<'a>() -> &'a str{
        "fantasy_team"
    }

    fn subscription_map_key(&self) -> Uuid{
        self.league_id
    }

    fn subscription_id_map(conn: &PgConn, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError>{
        Ok(publishables.iter().map(|c| (c.league_id, c.league_id)).collect())
    }
}

impl Publishable for Pick {

    fn message_type<'a>() -> &'a str{
        "pick"
    }

    fn subscription_map_key(&self) -> Uuid{
        self.pick_id
    }

    fn subscription_id_map(conn: &PgConn, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError>{
        let id_map = db::get_draft_ids_for_picks(&conn, &publishables.iter().map(|p| p.pick_id).collect())?;
        Ok(id_map.into_iter().collect())
    }
}

// impl Publishable for Pick {

//     fn message_type<'a>() -> &'a str{
//         "pick"
//     }

//     fn get_hierarchy_id(&self) -> Uuid{
//         self.pick_id
//     }

//     fn subscription_id(&self) -> Uuid{
//         // TODO FUCK!
//         self.pick
//     }
// }