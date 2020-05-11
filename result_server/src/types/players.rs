use serde::{Deserialize, Serialize};
use diesel_utils::{PgConn, DieselTimespan, my_timespan_format, my_timespan_format_opt};
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use frunk::labelled::transform_from;
use std::collections::HashMap;
use crate::publisher::Publishable;
use crate::diesel::RunQueryDsl;  // imported here so that can run db macros
use crate::diesel::ExpressionMethods;
use itertools::Itertools;
use crate::db;

#[derive(Insertable, Deserialize, LabelledGeneric, Queryable, Serialize, Debug)]
#[table_name = "players"]
pub struct Player {
    pub player_id: Uuid,
    pub meta: serde_json::Value,
}


#[derive(Serialize, Deserialize, Debug, LabelledGeneric, Clone)]
pub struct ApiPlayer{
    pub player_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<ApiPlayerName>,
    pub positions: Vec<ApiPlayerPosition>
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
                names: v.1,
                positions: v.2
            }
        })
        .collect_vec()
    }

    pub fn insert(conn: PgConn, players: Vec<Self>) -> Result<bool, diesel::result::Error>{
        let names: Vec<PlayerName> = players.clone().into_iter().flat_map(|t| {
            let player_id = t.player_id;
            t.names.into_iter().map(|n| {
                PlayerName{
                    player_name_id: Uuid::new_v4(), player_id, name: n.name, timespan: n.timespan
                }
            }).collect_vec()
        }).collect();
        insert_exec!(&conn, player_names::table, names)?;
        let raw_players: Vec<Player> = players.into_iter().map(transform_from).collect();
        insert_exec!(&conn, players::table, raw_players)?;
        Ok(true)
    }
}

impl Publishable for ApiPlayer {
    fn message_type<'a>() -> &'a str {
        "player"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.player_id
    }
}

impl Publishable for Player {
    fn message_type<'a>() -> &'a str {
        "player"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.player_id
    }
}

impl Publishable for PlayerUpdate {
    fn message_type<'a>() -> &'a str {
        "player_update"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.player_id
    }
}

impl Publishable for PlayerName {
    fn message_type<'a>() -> &'a str {
        "player_name"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.player_id
    }
}

impl Publishable for PlayerPosition {
    fn message_type<'a>() -> &'a str {
        "player_position"
    }

    fn get_hierarchy_id(&self) -> Uuid {
        self.player_id
    }
}