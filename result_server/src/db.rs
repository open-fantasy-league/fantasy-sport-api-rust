use crate::models::*;
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use uuid::Uuid;
use diesel::sql_types;
//use frunk::labelled::transform_from;
use itertools::Itertools;
use crate::handlers::ApiNewTeam;
use crate::DieselTimespan;


//sql_function!(fn trim_team_name_timespans(new_team_id sql_types::Uuid, new_timespan sql_types::Range<sql_types::Timestamptz>) -> ());
sql_function!(trim_team_name_timespans, WTF, (new_team_id: sql_types::Uuid, new_timespan: sql_types::Range<sql_types::Timestamptz>) -> ());

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

pub fn create_teams<'a>(conn: &PgConnection, new: Vec<ApiNewTeam>) -> Result<Vec<DbTeam>, diesel::result::Error>{
    use crate::schema::{teams, team_names};
    use crate::schema::teams::dsl as teams_col;
    use crate::schema::team_names::dsl as team_names_col;
    // TODO a nice-way to `From` one struct, into two structs
    // THis made vector-of-tuples, rather than tuple-of-vectors
    //let (new_db_teams, name_and_timespans): (Vec<DbNewTeam>, Vec<(String, DieselTimespan)>) = new.into_iter().map(|t| (DbNewTeam{team_id: t.team_id, meta: t.meta}, (t.name, t.timespan)).collect_vec();
    let length = new.len();
    let (new_db_teams, name_and_timespans): (Vec<DbNewTeam>, Vec<(String, DieselTimespan)>) = new
        .into_iter()
        .fold((Vec::with_capacity(length), Vec::with_capacity(length)), |(mut arr, mut arr2), t|{
            arr.push(DbNewTeam{team_id: t.team_id, meta: t.meta});
            arr2.push((t.name, t.timespan));
            (arr, arr2)
    });
    let teams_res = diesel::insert_into(teams::table).values(new_db_teams)
        .on_conflict(teams_col::team_id).do_update()
        .set(teams_col::meta.eq(excluded(teams_col::meta)))
        .get_results(conn);
    teams_res.map(|teams: Vec<DbTeam>|{
        let new_team_names: Vec<DbNewTeamName> = teams.iter().enumerate().map(|(i, t)| {
            let (new_name, new_timespan) = name_and_timespans[i].clone();
            trim_team_name_timespans(t.team_id, new_timespan);
            DbNewTeamName{team_id: t.team_id, name: new_name, timespan: new_timespan}
        }).collect_vec();

        diesel::insert_into(team_names::table).values(new_team_names).get_results::<DbTeamName>(conn);
        teams  // still want to just return original teams for now (to get new team-ids)
    })
}

pub fn create_players(conn: &PgConnection, new: Vec<DbNewPlayer>) -> Result<Vec<DbPlayer>, diesel::result::Error>{
    use crate::schema::players::{table, dsl::*};
    diesel::insert_into(table).values(&new)
        .on_conflict(player_id).do_update()
        .set(meta.eq(excluded(meta)))
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
