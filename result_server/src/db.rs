use crate::models::*;
use diesel::pg::PgConnection;
use diesel::RunQueryDsl;

pub fn create_competition<'a>(conn: &PgConnection, new: &DbNewCompetition) -> Result<DbCompetition, diesel::result::Error>{
    use crate::schema::competitions;
    diesel::insert_into(competitions::table).values(new).get_result(conn)
}

pub fn create_series<'a>(conn: &PgConnection, new: &DbNewSeries) -> Result<DbSeries, diesel::result::Error>{
    use crate::schema::series;
    diesel::insert_into(series::table).values(new).get_result(conn)
}

pub fn create_team<'a>(conn: &PgConnection, new: &DbNewTeam) -> Result<DbTeam, diesel::result::Error>{
    use crate::schema::teams;
    diesel::insert_into(teams::table).values(new).get_result(conn)
}
