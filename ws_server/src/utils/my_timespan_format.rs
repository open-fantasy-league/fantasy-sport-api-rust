// similar to https://serde.rs/custom-date-format.html
use chrono::{DateTime, Utc};
use serde::ser::SerializeSeq;
use serde::{self, de, Deserialize, Deserializer, Serializer};
use std::collections::Bound::{self, Excluded, Included};

pub type DieselTimespan = (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>);

pub fn deserialize<'de, D>(deserializer: D) -> Result<DieselTimespan, D::Error>
where
    D: Deserializer<'de>,
{
    // expecting "timespan": ["1984-blahblah", "1984-blah-blah"]
    // I couldnt find out how serde handled deserializing ranges/bounds,
    // but it seemed whatever it would be, would produce a que? from end-user,
    // when it's kind of natural to be inclusive both ends.
    // I wanted to deser to tuple, rather than vec. But seems more awkward,
    // I need to understand and create "visitors"
    //let parts: (DateTime::<Utc>, DateTime::<Utc>) = Deserializer::deserialize_tuple(2, deserializer)?;
    let parts = Vec::<DateTime<Utc>>::deserialize(deserializer)?;
    if parts.len() != 2 {
        // Seems like a should be able to just instantiate my serde-error,
        // rather than having to map it.....but dont know how
        Err("Must specify start and end of timespan. I.e. \"timespan\": [\"2019-08-15T17:41:18+00:00\", \"2019-08-15T17:41:18+00:00\"]").map_err(de::Error::custom)
    } else {
        let start = parts[0];
        let end = parts[1];
        Ok((Included(start), Excluded(end)))
    }
}

pub fn serialize<S>(timespan: &DieselTimespan, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let (start, end) = match timespan {
        (Included(dt0), Excluded(dt1)) => Ok((dt0, dt1)),
        _ => Err("Incorrect timerange format. THis should never happen!"),
    }
    .map_err(serde::ser::Error::custom)?;
    let mut seq = serializer.serialize_seq(Some(2))?;
    seq.serialize_element(start)?;
    seq.serialize_element(end)?;
    seq.end()
}