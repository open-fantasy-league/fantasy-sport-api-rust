use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::publisher::Publishable;
use diesel_utils::DieselTimespan;

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

    fn get_hierarchy_id(&self) -> Uuid{
        self.league_id
    }

    fn subscription_id(&self) -> Uuid{
        self.league_id
    }
}

impl Publishable for Pick {

    fn message_type<'a>() -> &'a str{
        "pick"
    }

    fn get_hierarchy_id(&self) -> Uuid{
        self.pick_id
    }

    fn subscription_id(&self) -> Uuid{
        // TODO FUCK!
        self.pick
    }
}