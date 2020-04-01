use super::schema::competitions;
use serde_json;
use chrono;
use std::collections::Bound;
use uuid::Uuid;

#[derive(Queryable)]
pub struct Competition {
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    //pub meta: Jsonb,
    //pub timespan: Range<Timestamptz>,
    pub timespan: (Bound<chrono::DateTime::<chrono::prelude::Utc>>, Bound<chrono::DateTime::<chrono::prelude::Utc>>),
}

#[derive(Insertable)]
#[table_name="competitions"]
pub struct NewCompetition<'a>{
    pub code: &'a str,
    pub name: &'a str,
    pub meta: serde_json::Value,
    pub timespan: (Bound<chrono::DateTime::<chrono::prelude::Utc>>, Bound<chrono::DateTime::<chrono::prelude::Utc>>),
}


#[derive(Queryable)]
pub struct Series {
    pub series_id: Uuid,
    pub code: String,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    pub timespan: (Bound<chrono::DateTime::<chrono::prelude::Utc>>, Bound<chrono::DateTime::<chrono::prelude::Utc>>),
}


