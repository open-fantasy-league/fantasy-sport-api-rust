use crate::models::*;
use diesel::pg::PgConnection;
use diesel::RunQueryDsl;
use serde_json::{Value, json};
use chrono::Utc;
use chrono::TimeZone;
use std::ops::Bound::Included;
use std::error;

//pub fn create_competition<'a>(
////    conn: &PgConnection, code: &'a str, name: &'a str, meta: Option<Value>,
////     start: chrono::DateTime::<chrono::prelude::Utc>, end: chrono::DateTime::<chrono::prelude::Utc>
////    ) -> Result<Competition, diesel::result::Error>{
pub fn create_competition<'a>(conn: &PgConnection, comp: &NewCompetition) -> Result<Competition, diesel::result::Error>{
    use crate::schema::competitions;
    //meta: meta.unwrap_or(json!({})),
    diesel::insert_into(competitions::table).values(comp).get_result(conn)
}
