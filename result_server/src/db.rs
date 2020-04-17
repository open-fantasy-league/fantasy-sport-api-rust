use crate::models::*;
use diesel::pg::PgConnection;
use diesel::pg::upsert::excluded;
use diesel::RunQueryDsl;
use diesel::ExpressionMethods;
use uuid::Uuid;
use diesel::{sql_types, sql_query};
//use frunk::labelled::transform_from;
use itertools::Itertools;
use crate::handlers::{ApiNewTeam, ApiNewPlayer};
use crate::DieselTimespan;

//sql_function! {fn coalesce<T: sql_types::NotNull>(a: sql_types::Nullable<T>, b: T) -> T;}
//sql_function!(fn trim_team_name_timespans(new_team_id sql_types::Uuid, new_timespan sql_types::Range<sql_types::Timestamptz>) -> ());
//sql_function!(trim_team_name_timespans, WTF, (new_team_id: sql_types::Uuid, new_timespan: sql_types::Range<sql_types::Timestamptz>) -> ());

fn trim_timespans(conn: &PgConnection, table_name: &str, id: Uuid, timespan: DieselTimespan) -> Result<usize, diesel::result::Error>{
    sql_query(format!("SELECT trim_{}_timespans($1, $2)", table_name))
    .bind::<sql_types::Uuid, _>(id)
    .bind::<sql_types::Range<sql_types::Timestamptz>, _>(timespan)
    .execute(conn)
}

// TODO macros for similar funcs
pub fn upsert_competitions<'a>(conn: &PgConnection, new: Vec<DbNewCompetition>) -> Result<Vec<DbCompetition>, diesel::result::Error>{
    use crate::schema::competitions::{table, dsl::*};
    // This "semi-upsert" doesnt work in postgres because it checks the inserts for null-ness, before other things,
    // so never fails the conflict check and goes into update part.
    // For now just do full upserts. fuck it.
    // let upsert_sql = "INSERT INTO competitions(competition_id, meta, name, timespan) VALUES (($1, $2, $3, $4), ($5, $6, $7, $8)) 
    //     ON CONFLICT DO UPDATE SET meta = coalesce(excluded.meta, meta), name = coalesce(excluded.name, name), timespan = coalesce(excluded.timespan, timespan)
    // "
    diesel::insert_into(table).values(new)
        .on_conflict(competition_id).do_update()
        .set((meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        //.set(name.eq(coalesce::<sql_types::Text>(excluded(name), name)))
        .get_results(conn)
}

pub fn upsert_serieses<'a>(conn: &PgConnection, new: Vec<DbNewSeries>) -> Result<Vec<DbSeries>, diesel::result::Error>{
    use crate::schema::series::{table, dsl::*};
    diesel::insert_into(table).values(new)
        .on_conflict(series_id).do_update()
        .set((competition_id.eq(excluded(competition_id)), meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

pub fn upsert_matches<'a>(conn: &PgConnection, new: Vec<DbNewMatch>) -> Result<Vec<DbMatch>, diesel::result::Error>{
    use crate::schema::matches::{table, dsl::*};
    diesel::insert_into(table).values(new)
        .on_conflict(match_id).do_update()
        .set((series_id.eq(excluded(series_id)), meta.eq(excluded(meta)), name.eq(excluded(name)), timespan.eq(excluded(timespan))))
        .get_results(conn)
}

pub fn upsert_teams<'a>(conn: &PgConnection, new: Vec<ApiNewTeam>) -> Result<Vec<DbTeam>, diesel::result::Error>{
    use crate::schema::{teams, team_names};
    use crate::schema::teams::dsl as teams_col;
    let num_entries = new.len();
    // TODO a nice-way to `From` one struct, into two structs.
    // Below was done in a way to avoid copies, and just move fields when needed.
    // However still have to clone when move out of vector, so felt a bit too high effort
    // for probably not even performance gain
    // just clone shit. who cares.
    /*let length = new.len();
    let (new_db_teams, name_and_timespans): (Vec<DbNewTeam>, Vec<(String, DieselTimespan)>) = new
        .into_iter()
        .fold((Vec::with_capacity(length), Vec::with_capacity(length)), |(mut arr, mut arr2), t|{
            arr.push(DbNewTeam{team_id: t.team_id, meta: t.meta});
            arr2.push((t.name, t.timespan));
            (arr, arr2)
    });*/
    let new_db_teams = new.iter().map(|t| DbNewTeam{team_id: t.team_id, meta: t.meta.clone()}).collect_vec();
    let teams_res = diesel::insert_into(teams::table).values(new_db_teams)
        .on_conflict(teams_col::team_id).do_update()
        .set(teams_col::meta.eq(excluded(teams_col::meta)))
        .get_results(conn);
    teams_res.and_then(|teams: Vec<DbTeam>|{
        let new_team_names = teams.iter().zip(new.into_iter()).map(|(t, n)| {
            let (new_name, new_timespan) = (n.name, n.timespan);
            trim_timespans(conn, "team_name", t.team_id, new_timespan)
            .map(|_| DbNewTeamName{team_id: t.team_id, name: new_name, timespan: new_timespan})
        })
        .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
            v.push(o);
            v
        });
        new_team_names.and_then(|nn|{
            diesel::insert_into(team_names::table).values(nn).execute(conn).map(|_| teams)
        })
    })
}

pub fn upsert_players(conn: &PgConnection, new: Vec<ApiNewPlayer>) -> Result<Vec<DbPlayer>, diesel::result::Error>{
    use crate::schema::{players, player_names};
    use crate::schema::players::dsl as players_col;
    let num_entries = new.len();
    let new_db_players = new.iter().map(|t| DbNewPlayer{player_id: t.player_id.clone(), meta: t.meta.clone()}).collect_vec();
    let players_res = diesel::insert_into(players::table).values(new_db_players)
        .on_conflict(players_col::player_id).do_update()
        .set(players_col::meta.eq(excluded(players_col::meta)))
        .get_results(conn);
    players_res.and_then(|players: Vec<DbPlayer>|{
        let new_player_names = players.iter().enumerate().map(|(i, t)| {
            let (new_name, new_timespan) = (new[i].name.clone(), new[i].timespan.clone());
            trim_timespans(conn, "player_name", t.player_id, new_timespan)
            .map(|_| DbNewPlayerName{player_id: t.player_id, name: new_name, timespan: new_timespan})
        })
        .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
            v.push(o);
            v
        });
        new_player_names.and_then(|nn|{
            diesel::insert_into(player_names::table).values(nn).execute(conn).map(|_| players)
        })
    })
}


pub fn upsert_series_teams<'a>(
        conn: &PgConnection, series_id: &Uuid, team_ids: &Vec<Uuid>
    ) -> Result<usize, diesel::result::Error>{
    use crate::schema::series_teams::{table, dsl};
    let values: Vec<_> = team_ids.iter()
        .map(|tid| (dsl::series_id.eq(series_id), dsl::team_id.eq(tid)))
        .collect();
    diesel::insert_into(table).values(&values).execute(conn)
}

pub fn upsert_team_players<'a>(
        conn: &PgConnection, team_players: Vec<DbNewTeamPlayer>
    ) -> Result<usize, diesel::result::Error>{
    use crate::schema::team_players::table;
    diesel::insert_into(table).values(&team_players).execute(conn)
}
