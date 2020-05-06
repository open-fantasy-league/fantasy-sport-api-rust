use serde::{Deserialize, Serialize};
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use warp_ws_server::utils::my_timespan_format_opt;
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;

#[derive(Queryable, LabelledGeneric, Serialize, Debug)]
pub struct Player {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(LabelledGeneric, Deserialize, Debug, Insertable)]
#[table_name = "players"]
pub struct NewPlayer {
    pub player_id: Option<Uuid>,
    pub meta: serde_json::Value
}

#[derive(Debug, LabelledGeneric, Queryable, Serialize, Identifiable, Associations)]
#[primary_key(player_name_id)]
#[belongs_to(Player)]
pub struct PlayerName {
    #[serde(skip_serializing)]
    player_name_id: Uuid,
    #[serde(skip_serializing)]
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Debug, Insertable, Deserialize, LabelledGeneric)]
#[table_name = "player_names"]
pub struct PlayerNameNew {
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(player_position_id)]
#[belongs_to(Player)]
pub struct PlayerPosition {
    #[serde(skip_serializing)]
    player_position_id: Uuid,
    pub player_id: Uuid,
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "player_positions"]
pub struct PlayerPositionIn {
    pub player_id: Uuid,
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Deserialize, Debug, LabelledGeneric)]
pub struct ApiPlayerName{
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApiPlayerPosition{
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(LabelledGeneric, Deserialize, Debug)]
pub struct ApiPlayerIn{
    pub player_id: Option<Uuid>,
    pub meta: serde_json::Value,
    pub names: Vec<ApiPlayerName>,
    pub positions: Vec<ApiPlayerPosition>
}

#[derive(Serialize, LabelledGeneric, Debug)]
pub struct ApiPlayerOut{
    pub player_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<ApiPlayerName>,
    pub positions: Vec<ApiPlayerPosition>
}