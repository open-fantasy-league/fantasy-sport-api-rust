use uuid::Uuid;
use serde::Deserialize;
use crate::types::{competitions::*, series::*, matches::*, players::*, teams::*, results::*};

#[derive(Deserialize, Debug)]
pub struct SubTeam{
    pub toggle: bool,
}
#[derive(Deserialize, Debug)]
pub struct SubCompetition{
    pub sub_competition_ids: Option<Vec<Uuid>>,
    pub unsub_competition_ids: Option<Vec<Uuid>>,
    pub all: Option<bool>
}

#[derive(Deserialize)]
#[serde(tag = "method")]
pub enum WSReq {
    Competition { message_id: Uuid, data: Vec<ApiCompetition>},
    CompetitionUpdate { message_id: Uuid, data: Vec<CompetitionUpdate>},
    Series { message_id: Uuid, data: Vec<ApiSeriesNew>},
    SeriesUpdate { message_id: Uuid, data: Vec<SeriesUpdate>},
    Match { message_id: Uuid, data: Vec<ApiMatchNew>},
    MatchUpdate { message_id: Uuid, data: Vec<MatchUpdate>},
    TeamSeriesResult { message_id: Uuid, data: Vec<TeamSeriesResult>},
    TeamSeriesResultUpdate { message_id: Uuid, data: Vec<TeamSeriesResultUpdate>},
    TeamMatchResult { message_id: Uuid, data: Vec<TeamMatchResult>},
    TeamMatchResultUpdate { message_id: Uuid, data: Vec<TeamMatchResultUpdate>},
    PlayerResult { message_id: Uuid, data: Vec<PlayerResult>},
    PlayerResultUpdate { message_id: Uuid, data: Vec<PlayerResultUpdate>},
    Team { message_id: Uuid, data: Vec<ApiTeam>},
    TeamUpdate { message_id: Uuid, data: Vec<TeamUpdate>},
    Player { message_id: Uuid, data: Vec<ApiPlayer>},
    PlayerUpdate { message_id: Uuid, data: Vec<PlayerUpdate>},
    TeamPlayer { message_id: Uuid, data: Vec<ApiTeamPlayer>},
    TeamName { message_id: Uuid, data: Vec<ApiTeamNameNew>},
    PlayerName { message_id: Uuid, data: Vec<ApiPlayerNameNew>},
    PlayerPosition { message_id: Uuid, data: Vec<ApiPlayerPositionNew>},
    SubTeam { message_id: Uuid, data: SubTeam},
    SubCompetition { message_id: Uuid, data: SubCompetition},
}