use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

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
    pub league_id: Option<Uuid>,
    pub external_user_id: Option<Uuid>,
    pub meta: Option<serde_json::Value>,
}