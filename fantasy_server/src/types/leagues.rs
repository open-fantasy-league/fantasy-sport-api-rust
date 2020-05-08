use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::publisher::Publishable;
use diesel_utils::{PgConn,DieselTimespan};
use std::collections::HashMap;
use crate::db;
use warp_ws_server::BoxError;
use chrono::{DateTime, Utc};


//https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(league_id)]
pub struct League {
    pub league_id: Uuid,
    pub name: String,
    pub team_size: i32,
    pub squad_size: i32,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    pub max_players_per_team: i32,
    pub max_players_per_position: i32,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "leagues"]
#[primary_key(league_id)]
pub struct LeagueUpdate {
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

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(League)]
#[table_name = "stat_multipliers"]
#[primary_key(league_id, name)]
pub struct StatMultiplier {
    pub league_id: Uuid,
    pub name: String,
    pub multiplier: f32,
    pub meta: serde_json::Value
}

#[derive(AsChangeset, Deserialize, Debug, Clone)]
#[table_name = "stat_multipliers"]
#[primary_key(league_id, name)]
pub struct StatMultiplierUpdate {
    pub league_id: Uuid,
    pub name: String,
    pub multiplier: Option<f32>,
    pub meta: Option<serde_json::Value>
}


#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(League)]
#[table_name = "periods"]
#[primary_key(period_id)]
pub struct Period {
    pub period_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    pub timespan: DieselTimespan,
    pub meta: serde_json::Value,
    pub points_multiplier: f32,
    pub teams_per_draft: i32,
    pub draft_interval_secs: i32,
    pub draft_start: DateTime<Utc>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "periods"]
#[primary_key(period_id)]
pub struct PeriodUpdate {
    pub period_id: Uuid,
    pub name: Option<String>,
    pub timespan: Option<DieselTimespan>,
    pub meta: Option<serde_json::Value>,
    pub points_multiplier: Option<f32>,
    pub teams_per_draft: Option<i32>,
    pub draft_interval_secs: Option<i32>,
    pub draft_start: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiLeague {
    pub league_id: Uuid,
    pub name: String,
    pub team_size: i32,
    pub squad_size: i32,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    pub max_players_per_team: i32,
    pub max_players_per_position: i32,
    pub periods: Vec<Period>,
    pub stat_multipliers: Vec<StatMultiplier>,
}

impl ApiLeague{
    pub fn from_rows(rows: Vec<(League, Vec<Period>, Vec<StatMultiplier>)>) -> Vec<Self>{
        rows.into_iter().map(|(l, periods, stats)|{
            Self{
                league_id: l.league_id, name: l.name, team_size: l.team_size, squad_size: l.squad_size, competition_id: l.competition_id,
                meta: l.meta, max_players_per_team: l.max_players_per_team, max_players_per_position: l.max_players_per_position,
                periods: periods, stat_multipliers: stats
            }
        }).collect()
    }
}

impl Publishable for League {

    fn message_type<'a>() -> &'a str{
        "league"
    }

    fn subscription_map_key(&self) -> Uuid{
        self.league_id
    }

    fn subscription_id_map(conn: &PgConn, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError>{
        Ok(publishables.iter().map(|c| (c.league_id, c.league_id)).collect())
    }
}

impl Publishable for Period {

    fn message_type<'a>() -> &'a str{
        "period"
    }

    fn subscription_map_key(&self) -> Uuid{
        self.league_id
    }

    fn subscription_id_map(conn: &PgConn, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError>{
        Ok(publishables.iter().map(|c| (c.league_id, c.league_id)).collect())
    }
}

impl Publishable for StatMultiplier {

    fn message_type<'a>() -> &'a str{
        "stat_multiplier"
    }

    fn subscription_map_key(&self) -> Uuid{
        self.league_id
    }

    fn subscription_id_map(conn: &PgConn, publishables: &Vec<Self>) -> Result<HashMap<Uuid, Uuid>, BoxError>{
        Ok(publishables.iter().map(|c| (c.league_id, c.league_id)).collect())
    }
}