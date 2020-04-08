use crate::models::*;
use diesel::pg::PgConnection;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use uuid::Uuid;

pub fn create_competition<'a>(conn: &PgConnection, new: &DbNewCompetition) -> Result<DbCompetition, diesel::result::Error>{
    use crate::schema::competitions;
    diesel::insert_into(competitions::table).values(new).get_result(conn)
}

pub fn create_series<'a>(conn: &PgConnection, new: &DbNewSeries) -> Result<DbSeries, diesel::result::Error>{
    use crate::schema::series;
    diesel::insert_into(series::table).values(new).get_result(conn)
}

pub fn create_match<'a>(conn: &PgConnection, new: &DbNewMatch) -> Result<DbMatch, diesel::result::Error>{
    use crate::schema::matches;
    diesel::insert_into(matches::table).values(new).get_result(conn)
}

pub fn create_team<'a>(conn: &PgConnection, new: &DbNewTeam) -> Result<DbTeam, diesel::result::Error>{
    use crate::schema::teams;
    diesel::insert_into(teams::table).values(new).get_result(conn)
}

pub fn create_player<'a>(conn: &PgConnection, new: &DbNewPlayer) -> Result<DbPlayer, diesel::result::Error>{
    use crate::schema::players;
    diesel::insert_into(players::table).values(new).get_result(conn)
}

/*pub fn create_series_team<'a>(conn: &PgConnection, series_id: &Uuid, team_id: &Uuid) -> Result<DbSeriesTeam, diesel::result::Error>{
    use crate::schema::series_teams::{table, dsl};
    diesel::insert_into(table).values((&dsl::series_id.eq(series_id), &dsl::team_id.eq(team_id))).get_result(conn)
}*/

pub fn create_series_teams<'a>(
        conn: &PgConnection, series_id: &Uuid, team_ids: Vec<Uuid>
    ) -> Result<usize, diesel::result::Error>{
    use crate::schema::series_teams::{table, dsl};
    let values: Vec<_> = team_ids.iter()
        .map(|tid| (dsl::series_id.eq(series_id), dsl::team_id.eq(tid)))
        .collect();
    diesel::insert_into(table).values(&values).execute(conn)
}

pub fn create_team_players<'a>(
        conn: &PgConnection, team_players: Vec<DbNewTeamPlayer>
    ) -> Result<usize, diesel::result::Error>{
    use crate::schema::team_players::table;
    diesel::insert_into(table).values(&team_players).execute(conn)
}
