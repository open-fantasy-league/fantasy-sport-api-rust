use crate::schema::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// If we hackily let valid-pick-ids be an array on period,
// Then to check if a pick was valid we'd have to O(n), as have to search through all
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(period_id, player_id)]
pub struct ValidPlayer {
    pub period_id: Uuid,
    pub player_id: Uuid,
}