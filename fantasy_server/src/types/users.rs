use crate::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use uuid::Uuid;
#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(external_user_id)]
pub struct ExternalUser {
    pub external_user_id: Uuid,
    pub username: String,
    pub meta: serde_json::Value,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "external_users"]
#[primary_key(external_user_id)]
pub struct ExternalUserUpdate {
    pub external_user_id: Uuid,
    pub username: Option<String>,
    pub meta: Option<serde_json::Value>,
}

#[derive(Insertable, Deserialize, Queryable, Serialize, Debug, Identifiable, Associations)]
#[primary_key(commissioner_id)]
pub struct Commissioner {
    pub commissioner_id: Uuid,
    pub external_user_id: Uuid,
    pub meta: serde_json::Value,
}

//http://diesel.rs/guides/all-about-updates/
#[derive(AsChangeset, Deserialize, Debug)]
#[table_name = "commissioners"]
#[primary_key(comissioner_ier)]
pub struct ComissionerUpdate {
    pub commissioner_id: Uuid,
    pub external_user_id: Option<Uuid>,
    pub meta: Option<serde_json::Value>,
}