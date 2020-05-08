use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
use crate::publisher::Publishable;
use diesel_utils::DieselTimespan;

//https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(draft_id)]
pub struct Draft {
    pub draft_id: Uuid,
    pub period_id: Uuid,
    pub meta: serde_json::Value,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "drafts"]
#[primary_key(draft_id)]
pub struct DraftUpdate {
    pub draft_id: Uuid,
    //pub period_id: Option<Uuid>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(team_draft_id)]
pub struct TeamDraft {
    pub team_draft_id: Uuid,
    pub draft_id: Uuid,
    pub fantasy_team_id: Uuid,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(draft_choice_id)]
pub struct DraftChoice {
    pub draft_choice_id: Uuid,
    pub team_draft_id: Uuid,
    pub timespan: DieselTimespan,
    pub pick_id: Option<Uuid>,
}

#[derive(AsChangeset, Deserialize, Debug)]
#[primary_key(draft_choice_id)]
#[table_name = "draft_choices"]
pub struct DraftChoiceUpdate {
    pub draft_choice_id: Uuid,
    // think this timespan wants to be mutable, if draft rescheduled or something
    pub timespan: Option<DieselTimespan>,
    pub pick_id: Option<Uuid>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations, AsChangeset)]
#[primary_key(fantasy_team_id)]
pub struct DraftQueue {
    pub fantasy_team_id: Uuid,
    pub player_ids: Vec<Uuid>,
}



impl Publishable for Draft{
    fn subscription_id(&self) -> Uuid{
        self.draft_id
    }

    fn message_type<'a>() -> &'a str{
        "draft"
    }

    fn get_hierarchy_id(&self) -> Uuid{
        self.draft_id
    }
}

// impl ApiDraft{
//     pub fn from_rows(rows: Vec<(League, Vec<Period>, Vec<StatMultiplier>)>) -> Vec<Self>{
//         rows.into_iter().map(|(l, periods, stats)|{
//             Self{
//                 league_id: l.league_id, name: l.name, team_size: l.team_size, squad_size: l.squad_size, competition_id: l.competition_id,
//                 meta: l.meta, teams_per_draft: l.teams_per_draft, max_players_per_team: l.max_players_per_team, max_players_per_position: l.max_players_per_position,
//                 periods: periods, stat_multipliers: stats
//             }
//         }).collect()
//     }
// }