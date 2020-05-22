use diesel_utils::{my_timespan_format, DieselTimespan};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//TODO how to "import" these from other crates?
// Dont think can use intermediate/3rd crate, because need the insertable/queryable diesel stuff
// which wont work without importing the schema.
#[derive(Deserialize, Serialize)]
#[serde(tag = "method")]
pub enum ResultMsgs {
    SubTeam {
        message_id: Uuid,
        data: ApiTeamsAndPlayers,
        mode: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiTeamsAndPlayers {
    pub teams: Vec<ApiTeam>,
    pub players: Vec<ApiPlayer>,
    pub team_players: Vec<TeamPlayer>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct TeamPlayer {
    #[serde(skip_serializing)]
    team_player_id: Uuid,
    pub team_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiTeamPlayer {
    pub team_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiTeam {
    pub team_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<ApiTeamName>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ApiPlayer {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<ApiPlayerName>,
    pub positions: Vec<ApiPlayerPosition>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiPlayerName {
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiPlayerPosition {
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ApiTeamName {
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}
