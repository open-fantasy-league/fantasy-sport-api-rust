use super::fantasy_teams::*;
use crate::schema::*;
use diesel_utils::{my_timespan_format, my_timespan_format_opt, DieselTimespan};
use frunk::LabelledGeneric;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;

//https://kotiri.com/2018/01/31/postgresql-diesel-rust-types.html
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(draft_id)]
pub struct Draft {
    pub draft_id: Uuid,
    pub period_id: Uuid,
    pub meta: serde_json::Value,
}

impl Draft {
    pub fn new(period_id: Uuid) -> Self {
        let meta = serde_json::json!({});
        let draft_id = Uuid::new_v4();
        Self {
            draft_id,
            period_id,
            meta,
        }
    }
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
#[belongs_to(Draft)]
#[belongs_to(FantasyTeam)]
#[primary_key(team_draft_id)]
pub struct TeamDraft {
    pub team_draft_id: Uuid,
    pub draft_id: Uuid,
    pub fantasy_team_id: Uuid,
}

impl TeamDraft {
    pub fn new(draft_id: Uuid, fantasy_team_id: Uuid) -> Self {
        let team_draft_id = Uuid::new_v4();
        Self {
            team_draft_id,
            draft_id,
            fantasy_team_id,
        }
    }

    pub fn test() -> Self {
        Self {
            team_draft_id: Uuid::new_v4(),
            fantasy_team_id: Uuid::new_v4(),
            draft_id: Uuid::new_v4(),
        }
    }
}

#[derive(
    Insertable,
    Deserialize,
    Queryable,
    Serialize,
    Debug,
    Identifiable,
    Associations,
    LabelledGeneric,
)]
#[primary_key(draft_choice_id)]
#[belongs_to(TeamDraft)]
pub struct DraftChoice {
    pub draft_choice_id: Uuid,
    pub team_draft_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
}

impl DraftChoice {
    pub fn new(team_draft_id: Uuid, timespan: DieselTimespan) -> Self {
        let draft_choice_id = Uuid::new_v4();
        Self {
            draft_choice_id,
            team_draft_id,
            timespan,
        }
    }
}

impl From<ApiDraftChoice> for DraftChoice {
    fn from(other: ApiDraftChoice) -> Self {
        Self {
            draft_choice_id: other.draft_choice_id,
            team_draft_id: other.team_draft_id,
            timespan: other.timespan,
        }
    }
}

#[derive(AsChangeset, Deserialize, Debug)]
#[primary_key(draft_choice_id)]
#[table_name = "draft_choices"]
pub struct DraftChoiceUpdate {
    pub draft_choice_id: Uuid,
    // think this timespan wants to be mutable, if draft rescheduled or something
    #[serde(with = "my_timespan_format_opt")]
    pub timespan: Option<DieselTimespan>,
}

#[derive(
    Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations, AsChangeset,
)]
#[primary_key(fantasy_team_id)]
pub struct DraftQueue {
    pub fantasy_team_id: Uuid,
    pub player_ids: Vec<Uuid>,
}

#[derive(Serialize, Debug, LabelledGeneric, Clone)]
pub struct ApiDraftChoice {
    pub draft_choice_id: Uuid,
    pub team_draft_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub fantasy_team_id: Uuid,
}

impl ApiDraftChoice {
    pub fn new(fantasy_team_id: Uuid, team_draft_id: Uuid, timespan: DieselTimespan) -> Self {
        let draft_choice_id = Uuid::new_v4();
        Self {
            fantasy_team_id,
            draft_choice_id,
            team_draft_id,
            timespan,
        }
    }

    // pub fn to_api(&self, fantasy_team_id: Uuid) -> ApiDraftChoice {
    //     ApiDraftChoice {
    //         draft_choice_id: self.draft_choice_id,
    //         team_draft_id: self.team_draft_id,
    //         timespan: self.timespan,
    //         pick_id: self.pick_id,
    //         fantasy_team_id,
    //     }
    // }
}

// #[derive(Serialize, Debug)]
// pub struct ApiTeamDraft {
//     pub team_draft_id: Uuid,
//     pub draft_id: Uuid,
//     pub fantasy_team_id: Uuid,
//     pub choices: Vec<DraftChoice>,
// }

// #[derive(Serialize, Debug)]
// pub struct ApiDraft {
//     pub league_id: Uuid,
//     pub draft_id: Uuid,
//     pub period_id: Uuid,
//     pub meta: serde_json::Value,
//     pub choices: Vec<ApiDraftChoice>, //pub teams: Vec<ApiTeamDraft>,
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiDraft {
    pub league_id: Uuid,
    pub draft_id: Uuid,
    pub period_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_drafts: Option<Vec<ApiTeamDraft>>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ApiTeamDraft {
    pub team_draft_id: Uuid,
    pub fantasy_team_id: Uuid,
    pub name: String,
    pub external_user_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_choices: Option<Vec<ApiDraftChoice2>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_picks: Option<Vec<ApiPick>>,
}

#[derive(Deserialize, Serialize, Debug, LabelledGeneric)]
pub struct ApiPick {
    pub pick_id: Uuid,
    pub player_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub draft_id: Option<Uuid>,
    pub fantasy_team_id: Option<Uuid>,
}

#[derive(Deserialize, Serialize, Debug, LabelledGeneric)]
pub struct ApiDraftChoice2 {
    pub draft_choice_id: Uuid,
    #[serde(with = "my_timespan_format")]
    pub timespan: DieselTimespan,
    pub pick: Option<Pick>,
}
