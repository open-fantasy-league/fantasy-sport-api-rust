use crate::schema::*;
use chrono::{DateTime, Utc};

use super::fantasy_teams::*;
use diesel_utils::{
    my_timespan_format, my_timespan_format_opt, new_dieseltimespan, DieselTimespan,
};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

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
    pub max_squad_players_same_team: i32,
    pub max_squad_players_same_position: i32,
    pub max_team_players_same_team: i32,
    pub max_team_players_same_position: i32,
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
    pub max_squad_players_same_team: Option<i32>,
    //pub max_squad_players_same_position: Option<i32>,
    pub max_team_players_same_team: Option<i32>,
    //pub max_team_players_same_position: Option<i32>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(League)]
#[table_name = "max_players_per_positions"]
#[primary_key(league_id, position)]
pub struct MaxPlayersPerPosition {
    pub league_id: Uuid,
    pub position: String,
    pub team_max: i32,
    pub squad_max: i32,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(League)]
#[table_name = "stat_multipliers"]
#[primary_key(league_id, name)]
pub struct StatMultiplier {
    pub league_id: Uuid,
    pub name: String,
    pub multiplier: f32,
    pub meta: serde_json::Value,
}

#[derive(AsChangeset, Deserialize, Debug, Clone)]
#[table_name = "stat_multipliers"]
#[primary_key(league_id, name)]
pub struct StatMultiplierUpdate {
    pub league_id: Uuid,
    pub name: String,
    pub multiplier: Option<f32>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(League)]
#[table_name = "periods"]
#[primary_key(period_id)]
pub struct Period {
    pub period_id: Uuid,
    pub league_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub meta: serde_json::Value,
    pub points_multiplier: f32,
    pub teams_per_draft: i32,
    pub draft_interval_secs: i32,
    pub draft_start: DateTime<Utc>,
    pub draft_lockdown: DateTime<Utc>,
}

impl Period {
    pub fn test() -> Self {
        Self {
            period_id: Uuid::new_v4(),
            league_id: Uuid::new_v4(),
            name: "Test".to_string(),
            timespan: new_dieseltimespan(Utc::now(), Utc::now()),
            meta: serde_json::json!({}),
            points_multiplier: 1.0,
            teams_per_draft: 5,
            draft_interval_secs: 10,
            draft_start: Utc::now(),
            draft_lockdown: Utc::now(),
        }
    }
}

#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "periods"]
#[primary_key(period_id)]
pub struct PeriodUpdate {
    pub period_id: Uuid,
    pub name: Option<String>,
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
    pub meta: Option<serde_json::Value>,
    pub points_multiplier: Option<f32>,
    pub teams_per_draft: Option<i32>,
    pub draft_interval_secs: Option<i32>,
    pub draft_start: Option<DateTime<Utc>>,
    pub draft_lockdown: Option<DateTime<Utc>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiLeague {
    pub league_id: Uuid,
    pub name: String,
    pub team_size: i32,
    pub squad_size: i32,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    pub max_squad_players_same_team: i32,
    pub max_team_players_same_team: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub periods: Option<Vec<Period>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stat_multipliers: Option<Vec<StatMultiplier>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fantasy_teams: Option<Vec<FantasyTeam>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_players_per_position: Option<Vec<MaxPlayersPerPosition>>,
}

impl ApiLeague {
    pub fn from_rows(
        rows: Vec<(
            League,
            Vec<Period>,
            Vec<StatMultiplier>,
            Vec<MaxPlayersPerPosition>,
            Vec<FantasyTeam>,
        )>,
    ) -> Vec<Self> {
        rows.into_iter()
            .map(
                |(l, periods, stats, max_players_per_position, fantasy_teams)| Self {
                    periods: Some(periods),
                    stat_multipliers: Some(stats),
                    fantasy_teams: Some(fantasy_teams),
                    max_players_per_position: Some(max_players_per_position),
                    league_id: l.league_id,
                    name: l.name,
                    team_size: l.team_size,
                    squad_size: l.squad_size,
                    competition_id: l.competition_id,
                    meta: l.meta,
                    max_squad_players_same_team: l.max_squad_players_same_team,
                    max_team_players_same_team: l.max_team_players_same_team,
                },
            )
            .collect()
    }

    pub fn from_leagues(leagues: Vec<League>) -> Vec<Self> {
        leagues
            .into_iter()
            .map(|l| Self {
                fantasy_teams: None,
                periods: None,
                stat_multipliers: None,
                max_players_per_position: None,
                league_id: l.league_id,
                name: l.name,
                team_size: l.team_size,
                squad_size: l.squad_size,
                competition_id: l.competition_id,
                meta: l.meta,
                max_squad_players_same_team: l.max_squad_players_same_team,
                max_team_players_same_team: l.max_team_players_same_team,
            })
            .collect()
    }
}
