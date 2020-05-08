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
    pub interval_secs: i32,
    pub period_id: Uuid,
    pub meta: serde_json::Value,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "drafts"]
#[primary_key(draft_id)]
pub struct DraftUpdate {
    pub draft_id: Uuid,
    pub interval_secs: Option<i32>,
    pub period_id: Option<Uuid>,
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

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(fantasy_team_id)]
pub struct DraftQueue {
    pub fantasy_team_id: Uuid,
    pub player_ids: Vec<Uuid>,
}


#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(pick_id)]
pub struct Pick {
    pub pick_id: Uuid,
    pub fantasy_team_id: Uuid,
    pub player_id: Uuid,
    pub timespan: DieselTimespan,
    pub active: bool,
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