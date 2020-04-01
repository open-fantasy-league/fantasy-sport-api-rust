use crate::models::*;
use diesel::pg::PgConnection;
use diesel::RunQueryDsl;
use serde_json;
use chrono::Utc;
use chrono::TimeZone;
use std::ops::Bound::{Included, Excluded, Unbounded};
use std::error::Error;

pub fn create_competition<'a>(conn: &PgConnection, code: &'a str, name: &'a str) -> Result<Competition, Box<dyn Error>>{
    use crate::schema::competitions;
    let meta = serde_json::from_str(r#"{}"#)?;//.unwrap();
    let new_competition = NewCompetition{
        code: code,
        name: name,
        meta: meta,
        timespan: (Included(Utc.ymd(1970, 1, 1).and_hms_milli(0, 0, 0, 0)), Unbounded),
    };

    diesel::insert_into(competitions::table).values(&new_competition).get_result(conn).map_err(|e| Box::new(e))
}
