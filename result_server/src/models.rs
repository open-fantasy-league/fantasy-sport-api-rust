use super::schema::*;
use crate::utils::my_timespan_format;
use crate::DieselTimespan;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Queryable, Serialize, Debug)]
pub struct DbCompetition {
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric, Debug)]
#[table_name = "competitions"]
pub struct DbNewCompetition {
    pub competition_id: Option<Uuid>,
    //pub name: &'a str, // This didnt work. think similar to https://stackoverflow.com/a/57977257/3920439
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbSeries {
    pub series_id: Uuid,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "series"]
pub struct DbNewSeries {
    pub series_id: Option<Uuid>,
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbMatch {
    pub match_id: Uuid,
    pub name: String,
    pub series_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric, Debug)]
#[table_name = "matches"]
pub struct DbNewMatch {
    pub match_id: Option<Uuid>,
    pub series_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbTeam {
    pub team_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbTeamName {
    #[serde(skip_serializing)]
    team_name_id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "teams"]
pub struct DbNewTeam {
    pub team_id: Option<Uuid>,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "team_names"]
pub struct DbNewTeamName {
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbPlayer {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbPlayerName {
    #[serde(skip_serializing)]
    player_name_id: Uuid,
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "players"]
pub struct DbNewPlayer {
    pub player_id: Option<Uuid>,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "player_names"]
pub struct DbNewPlayerName {
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Debug)]
pub struct DbSeriesTeam {
    pub series_id: Uuid,
    pub team_id: Uuid,
}

#[derive(Insertable, Deserialize)]
#[table_name = "series_teams"]
pub struct DbNewSeriesTeam {
    pub series_id: Uuid,
    pub team_id: Uuid,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, Debug)]
#[table_name = "team_players"]
pub struct DbNewTeamPlayer {
    pub team_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbTeamMatchResult {
    team_result_id: Uuid,
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}
#[derive(Insertable, Deserialize, Debug)]
#[table_name = "team_match_results"]
pub struct DbNewTeamMatchResult {
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbTeamSeriesResult {
    team_series_result_id: Uuid,
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}
#[derive(Insertable, Deserialize, Debug)]
#[table_name = "team_series_results"]
pub struct DbNewTeamSeriesResult {
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct DbPlayerMatchResult {
    player_result_id: Uuid,
    pub player_id: Uuid,
    pub match_id: Uuid,
    pub result: serde_json::Value,
    pub meta: serde_json::Value,
}
#[derive(Insertable, Deserialize, Debug)]
#[table_name = "player_results"]
pub struct DbNewPlayerMatchResult {
    pub player_id: Uuid,
    pub match_id: Uuid,
    pub result: serde_json::Value,
    pub meta: serde_json::Value,
}

pub trait Publishable {
    fn message_type<'a>() -> &'a str;
}

pub trait HasId {
    fn get_id(&self) -> Uuid;
}

impl HasId for DbTeamMatchResult {
    fn get_id(&self) -> Uuid {
        self.match_id
    }
}

impl Publishable for DbTeamMatchResult {
    fn message_type<'a>() -> &'a str {
        "team_match_results"
    }
}

impl HasId for DbPlayerMatchResult {
    fn get_id(&self) -> Uuid {
        self.match_id
    }
}

impl Publishable for DbPlayerMatchResult {
    fn message_type<'a>() -> &'a str {
        "player_match_results"
    }
}

impl HasId for DbTeamSeriesResult {
    fn get_id(&self) -> Uuid {
        self.series_id
    }
}

impl Publishable for DbTeamSeriesResult {
    fn message_type<'a>() -> &'a str {
        "team_series_results"
    }
}
