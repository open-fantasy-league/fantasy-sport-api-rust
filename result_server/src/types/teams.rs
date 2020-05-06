use serde::{Deserialize, Serialize};
use warp_ws_server::utils::my_timespan_format::{self, DieselTimespan};
use warp_ws_server::utils::my_timespan_format_opt;
use crate::schema::*;
use uuid::Uuid;
use serde_json;
use frunk::LabelledGeneric;
use std::collections::HashMap;
use super::{series::Series, players::*};
use itertools::Itertools;

#[derive(Queryable, Serialize, Debug)]
pub struct Team {
    pub team_id: Uuid,
    pub meta: serde_json::Value,
}

#[derive(Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(team_name_id)]
#[belongs_to(Team)]
pub struct TeamName {
    #[serde(skip_serializing)]
    team_name_id: Uuid,
    #[serde(skip_serializing)]
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

#[derive(Deserialize, LabelledGeneric, AsChangeset)]
#[primary_key(team_id)]
#[table_name = "teams"]
pub struct UpdateTeam {
    pub team_id: Uuid,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, LabelledGeneric)]
#[table_name = "team_names"]
pub struct NewTeamName {
    pub team_id: Uuid,
    pub name: String,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(Serialize, Debug)]
pub struct ApiTeamsAndPlayers{
    pub teams: Vec<ApiTeam>,
    pub players: Vec<ApiPlayerOut>,
    pub team_players: Vec<TeamPlayer>
}


#[derive(Deserialize, LabelledGeneric, Debug)]
pub struct ApiNewTeam{
    pub team_id: Option<Uuid>,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
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

#[derive(Serialize, Debug)]
pub struct ApiTeam{
    pub team_id: Uuid,
    pub meta: serde_json::Value,
    pub names: Vec<TeamName>,
}

impl ApiTeam{
    
    pub fn from_rows(rows: Vec<(Team, TeamName)>) -> Vec<Self>{
        // Group rows by team-id using hashmap, build a list of different team names
        // Assume if a team has no names ever, we dont care about it
        let mut acc: HashMap<Uuid, (Team, Vec<TeamName>)> = HashMap::new();
        acc = rows.into_iter().fold(acc, |mut acc, (team, team_name)| {
            match acc.get_mut(&team.team_id) {
                Some(t) => {t.1.push(team_name);},
                None => {acc.insert(team.team_id, (team, vec![team_name]));},
            }
            acc
        });

        acc.into_iter().map(|(team_id, v)|{
            Self{
                team_id: team_id,
                meta: v.0.meta,
                names: v.1
            }
        })
        .collect_vec()
    }
}