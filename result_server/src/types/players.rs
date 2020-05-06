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
pub struct PlayerNew {
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

// #[derive(Deserialize, LabelledGeneric, Debug)]
// pub struct ApiNewPlayer{
//     pub player_id: Option<Uuid>,
//     pub name: String,
//     pub meta: serde_json::Value,
//     #[serde(with = "my_timespan_format")]
//     // Its naive having initial player position and their name share a timespan,
//     // but fudge it! Improve later
//     pub timespan: DieselTimespan,
//     pub position: Option<String>
// }

// impl ApiPlayer{
    
//     pub fn from_rows(rows: Vec<(Player, PlayerName, PlayerPosition)>) -> Vec<Self>{
//         // Group rows by team-id using hashmap, build a list of different team names
//         // Assume if a team has no names ever, we dont care about it
//         let mut acc: HashMap<Uuid, (Player, Vec<PlayerName>, Vec<PlayerPosition>)> = HashMap::new();
//         acc = rows.into_iter().fold(acc, |mut acc, (player, player_name, position)| {
//             match acc.get_mut(&player.player_id) {
//                 Some(t) => {t.1.push(player_name); t.2.push(position)},
//                 None => {acc.insert(player.player_id, (player, vec![player_name], vec![position]));},
//             }
//             acc
//         });

//         acc.into_iter().map(|(pid, v)|{
//             Self{
//                 player_id: pid,
//                 meta: v.0.meta,
//                 names: v.1.into_iter().map(|tn| ApiPlayerName{name: tn.name, timespan: tn.timespan}).collect_vec()
//             }
//         })
//         .collect_vec()
//     }
// }