use diesel_utils::{my_timespan_format, DieselTimespan};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

//TODO how to "import" these from other crates?
// Dont think can use intermediate/3rd crate, because need the insertable/queryable diesel stuff
// which wont work without importing the schema.
#[derive(Deserialize, Serialize)]
#[serde(tag = "message_type")]
pub enum ResultMsgs {
    SubTeam {
        message_id: Uuid,
        data: Vec<ApiTeamWithPlayersHierarchy>,
        mode: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiTeamWithPlayersHierarchy {
    pub team_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<ApiTeamName>>,
    pub meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<ApiTeamPlayerOut>>,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ApiTeamPlayerOut {
    pub team_id: Uuid,
    pub player: ApiPlayer,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiTeamName {
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiPlayer {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<ApiPlayerName>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<ApiPlayerPosition>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiPlayerName {
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiPlayerPosition {
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}
