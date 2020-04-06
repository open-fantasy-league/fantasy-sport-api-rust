use super::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use chrono::{DateTime, Utc};
use std::collections::Bound;
use uuid::Uuid;
use crate::utils::my_timespan_format;

#[derive(Queryable, Serialize)]
pub struct DbCompetition {
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="competitions"]
pub struct DbNewCompetition{
    pub code: String,
    //pub name: &'a str, // This didnt work. think similar to https://stackoverflow.com/a/57977257/3920439
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct DbSeries {
    pub series_id: Uuid,
    pub code: String,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="series"]
pub struct DbNewSeries{
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct DbMatch {
    pub match_id: Uuid,
    pub series_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="matches"]
pub struct DbNewMatch{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct DbTeam {
    pub team_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="teams"]
pub struct DbNewTeam{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct DbPlayer {
    pub player_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="players"]
pub struct DbNewPlayer{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}


#[derive(Queryable)]
pub struct DbSeriesTeam {
    pub series_id: Uuid,
    pub team_id: Uuid,
}

#[derive(Insertable, Deserialize)]
#[table_name="series_teams"]
pub struct DbNewSeriesTeam{
    pub series_id: Uuid,
    pub team_id: Uuid,
}

