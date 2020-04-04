use super::schema::*;
use serde::{Deserialize, Serialize};
use serde_json;
use chrono::{DateTime, Utc, TimeZone};
use std::collections::Bound;
use uuid::Uuid;

#[derive(Queryable, Serialize)]
pub struct Competition {
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="competitions"]
pub struct NewCompetition{
    pub code: String,
    //pub name: &'a str, // This didnt work. think similar to https://stackoverflow.com/a/57977257/3920439
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct Series {
    pub series_id: Uuid,
    pub competition_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="series"]
pub struct NewSeries{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct Match {
    pub match_id: Uuid,
    pub series_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="matches"]
pub struct NewMatch{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct Team {
    pub team_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="teams"]
pub struct NewTeam{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Queryable, Serialize)]
pub struct Player {
    pub player_id: Uuid,
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}

#[derive(Insertable, Deserialize)]
#[table_name="players"]
pub struct NewPlayer{
    pub code: String,
    pub name: String,
    pub meta: serde_json::Value,
    #[serde(with = "my_timespan_format")]
    pub timespan: (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>),
}


#[derive(Queryable)]
pub struct SeriesTeam {
    pub series_id: Uuid,
    pub team_id: Uuid,
}

#[derive(Insertable, Deserialize)]
#[table_name="series_teams"]
pub struct NewSeriesTeam{
    pub series_id: Uuid,
    pub team_id: Uuid,
}


mod my_timespan_format{
    // similar to https://serde.rs/custom-date-format.html
    use serde::{self, de, Deserialize, Serializer, Deserializer};
    use serde::ser::{SerializeSeq};
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
        if parts.len() != 2{
            // Seems like a should be able to just instantiate my serde-error,
            // rather than having to map it.....but dont know how
            Err("Must specify start and end of timespan. I.e. \"timespan\": [\"2019-08-15T17:41:18+00:00\", \"2019-08-15T17:41:18+00:00\"]").map_err(de::Error::custom)
        }
        else{
            let start = parts[0];
            let end = parts[1];
            Ok((Included(start), Included(end)))
        }
    }

    pub fn serialize<S>(timespan: &(Bound<DateTime<Utc>>, Bound<DateTime<Utc>>), serializer: S) ->
        Result<S::Ok, S::Error> where S: Serializer,
        {
            let (start, end) = match timespan{
                (Included(dt0), Included(dt1)) => Ok((dt0, dt1)),
                _ => Err("Incorrect timerange format. THis should never happen!")
            }.map_err(serde::ser::Error::custom)?;
            let mut seq = serializer.serialize_seq(Some(2))?;
            seq.serialize_element(start)?;
            seq.serialize_element(end)?;
            seq.end()
        }
}




