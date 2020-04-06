use crate::models::*;
use diesel::pg::PgConnection;
use diesel::RunQueryDsl;

pub fn create_competition<'a>(conn: &PgConnection, comp: &DbNewCompetition) -> Result<DbCompetition, diesel::result::Error>{
    use crate::schema::competitions;
    //meta: meta.unwrap_or(json!({})),
    diesel::insert_into(competitions::table).values(comp).get_result(conn)
}

pub fn create_series<'a>(conn: &PgConnection, comp: &DbNewSeries) -> Result<DbSeries, diesel::result::Error>{
    use crate::schema::series;
    diesel::insert_into(series::table).values(comp).get_result(conn)
}
