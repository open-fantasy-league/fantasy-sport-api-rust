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
    pub max_players_per_team: i32,
    pub max_players_per_position: i32
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

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "leagues"]
#[primary_key(league_id)]
pub struct UpdateLeague {
    pub league_id: Uuid,
    pub name: Option<String>,
    // So for nullable fields, this wont let you set them back to null.
    // It's hard to support.
    // TODO test difference between missing fields and null in json
    pub meta: Option<serde_json::Value>,
    pub team_size: Option<i32>,
    pub squad_size: Option<i32>,
    pub competition_id: Option<Uuid>,
    // Think bug with
    /*
    If you wanted to assign NULL instead, you can either specify #[changeset_options(treat_none_as_null="true")] on the struct, 
    or you can have the field be of type Option<Option<T>>
    */
    // sending in "arg": null in json doesnt null it in db. It deserializes to None, rather than Some(None)
    // simpler to just make default a big number anyway. Then zero null-handling
    pub max_players_per_team: Option<i32>,
    pub max_players_per_position: Option<i32>
}