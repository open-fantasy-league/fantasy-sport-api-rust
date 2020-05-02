use super::schema::*;
use crate::utils::my_timespan_format;
use crate::DieselTimespan;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

#[derive(Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(competition_id)]
pub struct Competition {
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric, Debug)]
#[table_name = "competitions"]
pub struct NewCompetition {
    pub competition_id: Option<Uuid>,
    //pub name: &'a str, // This didnt work. think similar to https://stackoverflow.com/a/57977257/3920439
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug, Identifiable, Associations)]
#[belongs_to(Competition)]
#[primary_key(series_id)]
#[table_name = "series"]
pub struct Series {
    pub series_id: Uuid,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "series"]
pub struct NewSeries {
    pub series_id: Option<Uuid>,
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(match_id)]
#[table_name = "matches"]
pub struct Match {
    pub match_id: Uuid,
    pub name: String,
    pub series_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric, Debug)]
#[table_name = "matches"]
pub struct NewMatch {
    pub match_id: Option<Uuid>,
    pub series_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct Team {
    pub team_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct TeamName {
    #[serde(skip_serializing)]
    team_name_id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "teams"]
pub struct NewTeam {
    pub team_id: Option<Uuid>,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "team_names"]
pub struct NewTeamName {
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct Player {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct PlayerName {
    #[serde(skip_serializing)]
    player_name_id: Uuid,
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "players"]
pub struct NewPlayer {
    pub player_id: Option<Uuid>,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "player_names"]
pub struct NewPlayerName {
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Debug)]
pub struct SeriesTeam {
    pub series_id: Uuid,
    pub team_id: Uuid,
}

#[derive(Insertable, Deserialize)]
#[table_name = "series_teams"]
pub struct NewSeriesTeam {
    pub series_id: Uuid,
    pub team_id: Uuid,
}

#[derive(Insertable, Deserialize, Debug)]
#[table_name = "team_players"]
pub struct NewTeamPlayer {
    pub team_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct TeamPlayer {
    team_player_id: Uuid,
    pub team_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug)]
pub struct TeamMatchResult {
    team_result_id: Uuid,
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}
#[derive(Insertable, Deserialize, Debug)]
#[table_name = "team_match_results"]
pub struct NewTeamMatchResult {
    pub team_id: Uuid,
    pub match_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct TeamSeriesResult {
    team_series_result_id: Uuid,
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}
#[derive(Insertable, Deserialize, Debug)]
#[table_name = "team_series_results"]
pub struct NewTeamSeriesResult {
    pub team_id: Uuid,
    pub series_id: Uuid,
    pub result: String,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug)]
pub struct PlayerMatchResult {
    player_result_id: Uuid,
    pub player_id: Uuid,
    pub match_id: Uuid,
    pub result: serde_json::Value,
    pub meta: serde_json::Value,
}
#[derive(Insertable, Deserialize, Debug)]
#[table_name = "player_results"]
pub struct NewPlayerMatchResult {
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

impl HasId for TeamMatchResult {
    fn get_id(&self) -> Uuid {
        self.match_id
    }
}

impl Publishable for TeamMatchResult {
    fn message_type<'a>() -> &'a str {
        "team_match_results"
    }
}

impl HasId for PlayerMatchResult {
    fn get_id(&self) -> Uuid {
        self.match_id
    }
}

impl Publishable for PlayerMatchResult {
    fn message_type<'a>() -> &'a str {
        "player_match_results"
    }
}

impl HasId for TeamSeriesResult {
    fn get_id(&self) -> Uuid {
        self.series_id
    }
}

impl Publishable for TeamSeriesResult {
    fn message_type<'a>() -> &'a str {
        "team_series_results"
    }
}
