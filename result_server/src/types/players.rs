use serde::{Deserialize, Serialize};
use diesel_utils::{PgConn, DieselTimespan, my_timespan_format};
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use frunk::labelled::transform_from;
use std::collections::HashMap;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use itertools::Itertools;

#[derive(Insertable, Deserialize, LabelledGeneric, Queryable, Serialize, Debug, Identifiable)]
#[table_name = "players"]
#[primary_key(player_id)]
pub struct Player {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
}


#[derive(Serialize, Deserialize, Debug, LabelledGeneric, Clone)]
pub struct ApiPlayer{
    pub player_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<ApiPlayerName>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub positions: Option<Vec<ApiPlayerPosition>>
}

#[derive(Deserialize, Serialize, LabelledGeneric, AsChangeset, Debug)]
#[primary_key(player_id)]
#[table_name = "players"]
pub struct PlayerUpdate {
    pub player_id: Uuid,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Queryable, Deserialize, Serialize, Debug, Identifiable, Associations, LabelledGeneric)]
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

#[derive(Insertable, Queryable, Deserialize, Serialize, Debug, Identifiable, Associations, LabelledGeneric)]
#[primary_key(player_position_id)]
#[belongs_to(Player)]
pub struct PlayerPosition {
    #[serde(skip_serializing)]
    player_position_id: Uuid,
    #[serde(skip_serializing)]
    pub player_id: Uuid,
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

// #[derive(Deserialize, LabelledGeneric, AsChangeset)]
// #[primary_key(player_id)]
// #[table_name = "players"]
// pub struct UpdatePlayer {
//     pub player_id: Uuid,
//     pub meta: Option<serde_json::Value>,
// }

#[derive(Deserialize, Serialize, LabelledGeneric, Debug, Clone)]
pub struct ApiPlayerName {
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}


#[derive(Deserialize, Insertable, LabelledGeneric, Debug, Clone)]
#[table_name = "player_names"]
pub struct ApiPlayerNameNew {
    pub player_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Serialize, LabelledGeneric, Debug, Clone)]
pub struct ApiPlayerPosition {
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Deserialize, Insertable, LabelledGeneric, Debug, Clone)]
#[table_name = "player_positions"]
pub struct ApiPlayerPositionNew {
    pub player_id: Uuid,
    pub position: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

impl ApiPlayer{
    
    pub fn from_rows(rows: Vec<(Player, PlayerName, PlayerPosition)>) -> Vec<Self>{
        // Group rows by player-id using hashmap, build a list of different player names
        // Assume if a player has no names ever, we dont care about it
        let mut acc: HashMap<Uuid, (Player, Vec<ApiPlayerName>, Vec<ApiPlayerPosition>)> = HashMap::new();
        acc = rows.into_iter().fold(acc, |mut acc, (player, player_name, player_position)| {
            let player_name: ApiPlayerName = transform_from(player_name);
            let player_position: ApiPlayerPosition = transform_from(player_position);
            match acc.get_mut(&player.player_id) {
                Some(t) => {t.1.push(player_name);t.2.push(player_position)},
                None => {acc.insert(player.player_id, (player, vec![player_name], vec![player_position]));},
            }
            acc
        });

        acc.into_iter().map(|(player_id, v)|{
            Self{
                player_id: player_id,
                meta: v.0.meta,
                names: Some(v.1),
                positions: Some(v.2)
            }
        })
        .collect_vec()
    }


    pub fn from_diesel_rows(rows: Vec<(Player, Vec<PlayerName>, Vec<PlayerPosition>)>) -> Vec<Self>{
        rows.into_iter().map(|(p, names, positions)|{
            ApiPlayer{
                player_id: p.player_id,
                meta: p.meta,
                names: Some(names.into_iter().map(transform_from).collect_vec()),
                positions: Some(positions.into_iter().map(transform_from).collect_vec())
            }
        }).collect()
    }

    pub fn insert(conn: &PgConn, players: Vec<Self>) -> Result<bool, diesel::result::Error>{
        let names: Vec<PlayerName> = players.clone().into_iter().flat_map(|t| {
            let player_id = t.player_id;
            match t.names{
                Some(names) => {
                    names.into_iter().map(|n| {
                        PlayerName{
                            player_name_id: Uuid::new_v4(), player_id, name: n.name, timespan: n.timespan
                        }
                        }).collect_vec()
                },
                None => vec![]
            }
        }).collect();
        insert_exec!(conn, player_names::table, names)?;
        let raw_players: Vec<Player> = players.into_iter().map(transform_from).collect();
        insert_exec!(conn, players::table, raw_players)?;
        Ok(true)
    }
}