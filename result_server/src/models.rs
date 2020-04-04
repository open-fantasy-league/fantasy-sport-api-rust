use super::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use chrono::{DateTime, Utc, TimeZone};
use std::collections::Bound;
use uuid::Uuid;

#[derive(Queryable, Serialize, Deserialize)]
pub struct Competition {
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    //pub meta: Jsonb,
    //pub timespan: Range<Timestamptz>,
    #[serde(with = "my_timerange_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="competitions"]
pub struct NewCompetition{
    pub code: String,
    //pub name: &'a str, // This didnt work. think similar to https://stackoverflow.com/a/57977257/3920439
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timerange_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Player {
    pub player_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timerange_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize, Serialize)]
#[table_name="players"]
pub struct NewPlayer<'a>{
    pub code: &'a str,
    pub name: &'a str,
    pub meta: serde_json::Value,
    #[serde(with = "my_timerange_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}


#[derive(Queryable)]
pub struct Series {
    pub series_id: Uuid,
    pub code: String,
    pub name: String,
    pub competition_id: Uuid,
    pub meta: serde_json::Value,
    #[serde(with = "my_timerange_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}


mod my_timerange_format{
    // similar to https://serde.rs/custom-date-format.html
    use serde::{self, Deserialize, Serializer, Deserializer};
    use chrono::{DateTime, Utc};
    use std::collections::Bound::{self, Included};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<(Bound<DateTime<Utc>>, Bound<DateTime<Utc>>), D::Error> where D: Deserializer<'de>{
        // expecting "timespan": ["1984-blahblah", "1984-blah-blah"]
        // I couldnt find out how serde handled deserializing ranges/bounds,
        // but it seemed whatever it would be, would produce a que? from end-user,
        // when it's kind of natural to be inclusive both ends.
        // I wanted to deser to tuple, rather than vec. But seems more awkward,
        // I need to understand and create "visitors"
        //let parts: (DateTime::<Utc>, DateTime::<Utc>) = Deserializer::deserialize_tuple(2, deserializer)?;
        let parts = Vec::<DateTime::<Utc>>::deserialize(deserializer)?;
        let start = parts[0];
        let end = parts[1];
        Ok((Included(start), Included(end)))
    }
}




