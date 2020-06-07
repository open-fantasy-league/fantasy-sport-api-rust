use super::{drafts::*, leagues::*};
use crate::schema::*;
use diesel_utils::{my_timespan_format, DieselTimespan};
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

//https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(fantasy_team_id)]
#[belongs_to(League)]
pub struct FantasyTeam {
    pub fantasy_team_id: Uuid,
    pub name: String,
    pub league_id: Uuid,
    pub external_user_id: Uuid,
    pub meta: serde_json::Value,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "fantasy_teams"]
#[primary_key(fantasy_team_id)]
pub struct FantasyTeamUpdate {
    pub fantasy_team_id: Uuid,
    pub name: Option<String>,
    //pub league_id: Option<Uuid>,
    pub external_user_id: Option<Uuid>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(pick_id)]
#[belongs_to(FantasyTeam)]
#[belongs_to(DraftChoice)]
pub struct Pick {
    pub pick_id: Uuid,
    // having fantasy_team_id on pick is kind of duplicating data, it can be accessed through the draft_choice_id,
    // but you have to jump a couple of joins for that. THink its worth having fantasy-team-id here as well.
    // I think so long as the fields are immutable then duplication is ok
    pub fantasy_team_id: Uuid,
    pub draft_choice_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

// It's a bit foolish to ask client to specify which draft-choice this pick is for...clearly it's for the next one.
// Really this should be what the api-gateway/server is for.
// To take a client friendly request (i.e. pick this guy), and translate it into simple CRUD for fantasy_server
// (as well as maybe coordinate valid-player-id handling)
#[derive(Deserialize, Serialize, Debug)]
pub struct DraftPick {
    pub player_id: Uuid,
    pub fantasy_team_id: Uuid,
    pub draft_id: Uuid,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[primary_key(pick_id)]
#[table_name = "picks"]
pub struct PickUpdate {
    pub pick_id: Uuid,
    // These shouldnt be mutable
    // pub fantasy_team_id: Uuid,
    // pub player_id: Uuid,
    pub timespan: DieselTimespan,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(active_pick_id)]
#[belongs_to(Pick)]
pub struct ActivePick {
    pub active_pick_id: Uuid,
    pub pick_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[primary_key(active_pick_id)]
#[table_name = "active_picks"]
pub struct ActivePickUpdate {
    pub active_pick_id: Uuid,
    pub timespan: DieselTimespan,
}
