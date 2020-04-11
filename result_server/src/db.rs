use crate::models::*;
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use uuid::Uuid;

// TODO macros for similar funcs
pub fn create_competitions<'a>(conn: &PgConnection, new: Vec<DbNewCompetition>) -> Result<Vec<DbCompetition>, diesel::result::Error>{
    use crate::schema::competitions::{table, dsl::*};
    diesel::insert_into(table).values(new)
        .on_conflict(competition_id).do_update()
        // Investigate how to specify set as struct, rather than listing out all fields
        .set((meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

pub fn create_serieses<'a>(conn: &PgConnection, new: Vec<DbNewSeries>) -> Result<Vec<DbSeries>, diesel::result::Error>{
    use crate::schema::series::{table, dsl::*};
    diesel::insert_into(table).values(new)
        .on_conflict(series_id).do_update()
        .set((competition_id.eq(excluded(competition_id)), meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

pub fn create_matches<'a>(conn: &PgConnection, new: Vec<DbNewMatch>) -> Result<Vec<DbMatch>, diesel::result::Error>{
    use crate::schema::matches::{table, dsl::*};
    diesel::insert_into(table).values(new)
        .on_conflict(match_id).do_update()
        .set((series_id.eq(excluded(series_id)), meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

pub fn create_teams<'a>(conn: &PgConnection, new: Vec<DbNewTeam>) -> Result<Vec<DbTeam>, diesel::result::Error>{
    use crate::schema::teams::{table, dsl::*};
    diesel::insert_into(table).values(new)
        .on_conflict(team_id).do_update()
        .set((meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

pub fn create_players(conn: &PgConnection, new: Vec<DbNewPlayer>) -> Result<Vec<DbPlayer>, diesel::result::Error>{
    use crate::schema::players::{table, dsl::*};
    diesel::insert_into(table).values(&new)
        .on_conflict(player_id).do_update()
        .set((meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

/*pub fn create_series_team<'a>(conn: &PgConnection, series_id: &Uuid, team_id: &Uuid) -> Result<DbSeriesTeam, diesel::result::Error>{
    use crate::schema::series_teams::{table, dsl};
    diesel::insert_into(table).values((&dsl::series_id.eq(series_id), &dsl::team_id.eq(team_id))).get_result(conn)
}*/

pub fn create_series_teams<'a>(
        conn: &PgConnection, series_id: &Uuid, team_ids: &Vec<Uuid>
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
