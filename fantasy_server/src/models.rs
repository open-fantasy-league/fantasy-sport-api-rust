use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

//https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(league_id)]
pub struct League {
    pub league_id: Uuid,
    pub name: String,
    pub team_size: i32,
    pub squad_size: i32,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    pub max_players_per_team: Option<i32>,
    pub max_players_per_position: Option<i32>
}

#[derive(Insertable, Deserialize, Debug)]
#[table_name = "leagues"]
pub struct NewLeague {
    pub league_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    pub team_size: i32,
    pub squad_size: i32,
    pub competition_id: Uuid,
    pub max_players_per_team: Option<i32>,
    pub max_players_per_position: Option<i32>
}