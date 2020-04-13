use super::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use crate::DieselTimespan;
use uuid::Uuid;
use crate::utils::my_timespan_format;

#[derive(Queryable, Serialize)]
pub struct DbCompetition {
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="competitions"]
pub struct DbNewCompetition{
    pub competition_id: Option<Uuid>,
    //pub name: &'a str, // This didnt work. think similar to https://stackoverflow.com/a/57977257/3920439
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize)]
pub struct DbSeries {
    pub series_id: Uuid,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="series"]
pub struct DbNewSeries{
    pub series_id: Option<Uuid>,
    pub competition_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize)]
pub struct DbMatch {
    pub match_id: Uuid,
    pub name: String,
    pub series_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="matches"]
pub struct DbNewMatch{
    pub match_id: Option<Uuid>,
    pub series_id: Uuid,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize)]
pub struct DbTeam {
    pub team_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize)]
pub struct DbTeamName {
    #[serde(skip_serializing)]
    team_name_id: Uuid,
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="teams"]
pub struct DbNewTeam{
    pub team_id: Option<Uuid>,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="team_names"]
pub struct DbNewTeamName{
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize)]
pub struct DbPlayer {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize)]
pub struct DbPlayerName {
    #[serde(skip_serializing)]
    player_name_id: Uuid,
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="players"]
pub struct DbNewPlayer{
    pub player_id: Option<Uuid>,
    pub meta: serde_json::Value,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name="player_names"]
pub struct DbNewPlayerName{
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
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

#[derive(Insertable, Deserialize)]
#[table_name="team_players"]
pub struct DbNewTeamPlayer{
    pub team_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}
