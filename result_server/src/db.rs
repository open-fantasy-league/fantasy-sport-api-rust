#![macro_use]
use diesel::pg::expression::dsl::any;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use uuid::Uuid;
use frunk::labelled::transform_from;
use itertools::{izip, Itertools};
use warp_ws_server::utils::my_timespan_format::DieselTimespan;
use crate::types::{competitions::*, series::*, matches::*, teams::*, results::*, players::*};
use crate::schema;

//sql_function! {fn coalesce<T: sql_types::NotNull>(a: sql_types::Nullable<T>, b: T) -> T;}
//sql_function!(fn trim_team_name_timespans(new_team_id sql_types::Uuid, new_timespan sql_types::Range<sql_types::Timestamptz>) -> ());
sql_function!(trim_team_name_timespans, WTF, (new_team_id: sql_types::Uuid, new_timespan: sql_types::Range<sql_types::Timestamptz>) -> TeamName);

macro_rules! insert {
    ($conn:expr, $table:expr, $aggregate:expr) => {
        diesel::insert_into($table)
            .values($aggregate)
            .get_results($conn);
    };
}

macro_rules! insert_exec {
    ($conn:expr, $table:expr, $aggregate:expr) => {
        diesel::insert_into($table)
            .values($aggregate)
            .execute($conn);
    };
}

macro_rules! update {
    ($conn:expr, $table_name:ident, $pkey:ident, $changeset:expr) => {
        diesel::update(schema::$table_name::table).filter(schema::$table_name::dsl::$pkey.eq($changeset.$pkey))
            .set($changeset)
            .get_result($conn);
    };
}

macro_rules! update_2pkey {
    ($conn:expr, $table_name:ident, $pkey:ident, $pkey2:ident, $changeset:expr) => {
        diesel::update(schema::$table_name::table).filter(schema::$table_name::dsl::$pkey.eq($changeset.$pkey))
            .filter(schema::$table_name::dsl::$pkey2.eq($changeset.$pkey2))
            .set($changeset)
            .get_result($conn);
    };
}

//sql_function!(trim_team_name_timespans, TrimTeamNameTimespan, (x: sql_types::Uuid, sql_types::Range<sql_types::Timestamptz>) -> Vec<TeamName>);

// Fuck making this generic is hard
pub fn trim_timespans_team_name(
    conn: &PgConnection,
    table_name: &str,
    new: &Vec<ApiTeamNameNew>
) -> Result<Vec<usize>, diesel::result::Error>{
    let num_entries = new.len();
    let trimmed: Vec<_> = new.iter().map(|n|{
        sql_query(format!("SELECT * FROM trim_{}_timespans($1, $2)", table_name))
            .bind::<sql_types::Uuid, _>(n.team_id)
            .bind::<sql_types::Range<sql_types::Timestamptz>, _>(n.timespan)
            .execute(conn)
        
    })
    .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
                        v.push(o);
                        v
    })?;//.into_iter().flatten().collect();  // TODO is this really optimal?
    Ok(trimmed)
}

pub fn trim_timespans_team_player(
    conn: &PgConnection,
    table_name: &str,
    new: &Vec<ApiTeamPlayer>
) -> Result<Vec<usize>, diesel::result::Error>{
    let num_entries = new.len();
    let trimmed: Vec<_> = new.iter().map(|n|{
        sql_query(format!("SELECT trim_{}_timespans($1, $2)", table_name))
            .bind::<sql_types::Uuid, _>(n.team_id)
            .bind::<sql_types::Range<sql_types::Timestamptz>, _>(n.timespan)
            .execute(conn)
    })
    .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
                        v.push(o);
                        v
    })?;//.into_iter().flatten().collect();  // TODO is this really optimal?
    Ok(trimmed)
}

pub fn trim_timespans_player_name(
    conn: &PgConnection,
    table_name: &str,
    new: &Vec<ApiPlayerNameNew>
) -> Result<Vec<usize>, diesel::result::Error>{
    let num_entries = new.len();
    let trimmed: Vec<_> = new.iter().map(|n|{
        sql_query(format!("SELECT trim_{}_timespans($1, $2)", table_name))
            .bind::<sql_types::Uuid, _>(n.player_id)
            .bind::<sql_types::Range<sql_types::Timestamptz>, _>(n.timespan)
            .execute(conn)
    })
    .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
                        v.push(o);
                        v
    })?;//.into_iter().flatten().collect();  // TODO is this really optimal?
    Ok(trimmed)
}

pub fn trim_timespans_player_position(
    conn: &PgConnection,
    table_name: &str,
    new: &Vec<ApiPlayerPositionNew>
) -> Result<Vec<usize>, diesel::result::Error>{
    let num_entries = new.len();
    let trimmed: Vec<_> = new.iter().map(|n|{
        sql_query(format!("SELECT trim_{}_timespans($1, $2)", table_name))
            .bind::<sql_types::Uuid, _>(n.player_id)
            .bind::<sql_types::Range<sql_types::Timestamptz>, _>(n.timespan)
            .execute(conn)
    })
    .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
                        v.push(o);
                        v
    })?;//.into_iter().flatten().collect();  // TODO is this really optimal?
    Ok(trimmed)
}

// TODO to improve genericness of trimmed-timespans
// would prob help to pass trim postgresql func a vector 

// TODO maybe move these funcs onto struct::insert
pub async fn insert_team_names(
    conn: &PgConnection,
    new: Vec<ApiTeamNameNew>,
) -> Result<Vec<TeamName>, diesel::result::Error> {
    use crate::schema::team_names;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let trimmed: Vec<_> = trim_timespans_team_name(conn, "team_name", &new)?;
    //let trimmed: Vec<TeamName> = trim_timespans_many::<ApiTeamNameNew, TeamName>(conn, "team_name", new)?;
    let inserted: Vec<TeamName> = insert!(conn, team_names::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub async fn insert_player_names(
    conn: &PgConnection,
    new: Vec<ApiPlayerNameNew>,
) -> Result<Vec<PlayerName>, diesel::result::Error> {
    use crate::schema::player_names;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let num_entries = new.len();
    let trimmed: Vec<_> = trim_timespans_player_name(conn, "player_name", &new)?;
    let inserted: Vec<PlayerName> = insert!(conn, player_names::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub async fn insert_player_positions(
    conn: &PgConnection,
    new: Vec<ApiPlayerPositionNew>,
) -> Result<Vec<PlayerPosition>, diesel::result::Error> {
    use crate::schema::player_positions;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let num_entries = new.len();
    let trimmed: Vec<_> = trim_timespans_player_position(conn, "player_position", &new)?;
    let inserted: Vec<PlayerPosition> = insert!(conn, player_positions::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub async fn insert_team_players(
    conn: &PgConnection,
    new: Vec<ApiTeamPlayer>,
) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
    use crate::schema::team_players;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let num_entries = new.len();
    let trimmed: Vec<_> = trim_timespans_team_player(conn, "team_player", &new)?;
    let inserted: Vec<TeamPlayer> = insert!(conn, team_players::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}



// pub fn upsert_serieses<'a>(
//     conn: &PgConnection,
//     new: Vec<NewSeries>,
// ) -> Result<Vec<Series>, diesel::result::Error> {
//     use crate::schema::series::{dsl::*, table};
//     diesel::insert_into(table)
//         .values(new)
//         .on_conflict(series_id)
//         .do_update()
//         .set((
//             competition_id.eq(excluded(competition_id)),
//             meta.eq(excluded(meta)),
//             name.eq(excluded(name)),
//             timespan.eq(excluded(timespan)),
//         ))
//         .get_results(conn)
// }

// pub fn upsert_matches(
//     conn: &PgConnection,
//     new: Vec<NewMatch>,
// ) -> Result<Vec<Match>, diesel::result::Error> {
//     use crate::schema::matches::{dsl::*, table};
//     diesel::insert_into(table)
//         .values(new)
//         .on_conflict(match_id)
//         .do_update()
//         .set((
//             series_id.eq(excluded(series_id)),
//             meta.eq(excluded(meta)),
//             name.eq(excluded(name)),
//             timespan.eq(excluded(timespan)),
//         ))
//         .get_results(conn)
// }

// pub fn upsert_teams<'a>(
//     conn: &PgConnection,
//     new: Vec<ApiNewTeam>,
// ) -> Result<Vec<Team>, diesel::result::Error> {
//     use crate::schema::teams::dsl as teams_col;
//     use crate::schema::{team_names, teams};
//     let num_entries = new.len();
//     // TODO a nice-way to `From` one struct, into two structs.
//     // Below was done in a way to avoid copies, and just move fields when needed.
//     // However still have to clone when move out of vector, so felt a bit too high effort
//     // for probably not even performance gain
//     // just clone shit. who cares.
//     /*let length = new.len();
//     let (new_db_teams, name_and_timespans): (Vec<NewTeam>, Vec<(String, DieselTimespan)>) = new
//         .into_iter()
//         .fold((Vec::with_capacity(length), Vec::with_capacity(length)), |(mut arr, mut arr2), t|{
//             arr.push(NewTeam{team_id: t.team_id, meta: t.meta});
//             arr2.push((t.name, t.timespan));
//             (arr, arr2)
//     });*/
//     let new_db_teams = new
//         .iter()
//         .map(|t| NewTeam {
//             team_id: t.team_id,
//             meta: t.meta.clone(),
//         })
//         .collect_vec();
//     let teams_res = diesel::insert_into(teams::table)
//         .values(new_db_teams)
//         .on_conflict(teams_col::team_id)
//         .do_update()
//         .set(teams_col::meta.eq(excluded(teams_col::meta)))
//         .get_results(conn);
//     teams_res.and_then(|teams: Vec<Team>| {
//         let new_team_names = teams
//             .iter()
//             .zip(new.into_iter())
//             .map(|(t, n)| {
//                 let (new_name, new_timespan) = (n.name, n.timespan);
//                 trim_timespans(conn, "team_name", t.team_id, new_timespan).map(|_| NewTeamName {
//                     team_id: t.team_id,
//                     name: new_name,
//                     timespan: new_timespan,
//                 })
//             })
//             .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
//                 v.push(o);
//                 v
//             });
//         new_team_names.and_then(|nn| {
//             diesel::insert_into(team_names::table)
//                 .values(nn)
//                 .execute(conn)
//                 .map(|_| teams)
//         })
//     })
// }

// pub fn insert_players(
//     conn: &PgConnection,
//     new: Vec<ApiPlayerIn>,
// ) -> Result<Vec<Player>, diesel::result::Error> {
//     use crate::schema::{player_names, players, player_positions};
//     let num_entries = new.len();
//     // let new_db_players = new
//     //     .iter()
//     //     .map(|t| NewPlayer {
//     //         player_id: t.player_id,
//     //         meta: t.meta,
//     //     })
//     //     .collect_vec();
//     let new_db_players: Vec<PlayerIn> = new.into_iter().map(transform_from).collect_vec();
//     let player_res: Vec<Player> = insert!(conn, players::table, new_db_players)?;
//     let player_names_in = player_res.iter().zip(new).flat_map(|(pr, pn)|{
//         pn.names.iter().map(|x| PlayerNameIn{player_id: pr.player_id, name: x.name, timespan: x.timespan}).collect_vec()
//     }).collect_vec();
//     // COulkd do in one pass if nessers
//     let player_positions_in = player_res.iter().zip(new).flat_map(|(pr, pn)|{
//         pn.positions.iter().map(|x| PlayerPositionIn{player_id: pr.player_id, position: x.position, timespan: x.timespan}).collect_vec()
//     }).collect_vec();
//     let player_names = insert_exec!(conn, player_names::table, player_names_in)?;
//     let player_positions = insert_exec!(conn, player_positions::table, player_positions_in)?;
//     Ok(player_res)
// }

// // pub fn update_players(
// //     conn: &PgConnection,
// //     new: Vec<ApiNewPlayer>,
// // ) -> Result<Vec<Player>, diesel::result::Error> {
// //     use crate::schema::players::dsl as players_col;
// //     use crate::schema::{player_names, players};
// //     let num_entries = new.len();
// //     let new_db_players = new
// //         .iter()
// //         .map(|t| NewPlayer {
// //             player_id: t.player_id.clone(),
// //             meta: t.meta.clone(),
// //         })
// //         .collect_vec();
// //     let players_res = diesel::insert_into(players::table)
// //         .values(new_db_players)
// //         .on_conflict(players_col::player_id)
// //         .do_update()
// //         .set(players_col::meta.eq(excluded(players_col::meta)))
// //         .get_results(conn);
// //     players_res.and_then(|players: Vec<Player>| {
// //         let (new_player_names, new_player_positions) = players
// //             .iter()
// //             .enumerate()
// //             .map(|(i, t)| {
// //                 let (new_name, new_timespan, new_position) = (new[i].name.clone(), new[i].timespan.clone());
// //                 trim_timespans(conn, "player_name", t.player_id, new_timespan).map(|_| {
// //                     NewPlayerName {
// //                         player_id: t.player_id,
// //                         name: new_name,
// //                         timespan: new_timespan,
// //                     }
// //                 })
// //             })
// //             .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
// //                 v.push(o);
// //                 v
// //             });
// //         new_player_names.and_then(|nn| {
// //             diesel::insert_into(player_names::table)
// //                 .values(nn)
// //                 .execute(conn)
// //                 .map(|_| players)
// //         })
// //     })
// // }

// pub fn upsert_series_teams<'a>(
//     conn: &PgConnection,
//     series_id: &Uuid,
//     team_ids: &Vec<Uuid>,
// ) -> Result<usize, diesel::result::Error> {
//     use crate::schema::series_teams::{dsl, table};
//     let values: Vec<_> = team_ids
//         .iter()
//         .map(|tid| (dsl::series_id.eq(series_id), dsl::team_id.eq(tid)))
//         .collect();
//     diesel::insert_into(table).values(&values).execute(conn)
// }

// pub fn upsert_team_players<'a>(
//     conn: &PgConnection,
//     new: Vec<NewTeamPlayer>,
// ) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
//     use crate::schema::team_players;
//     let num_entries = new.len();
//     new.iter()
//         .map(|n| {
//             // map looks useless but want to pass our insertable onto fold-results OK part
//             trim_timespans(conn, "team_player", n.player_id, n.timespan).map(|_| n)
//         })
//         .fold_results(Vec::with_capacity(num_entries), |mut v, o| {
//             v.push(o);
//             v
//         })
//         .and_then(|nn| {
//             diesel::insert_into(team_players::table)
//                 .values(nn)
//                 .get_results(conn)
//         })
// }

// pub fn upsert_team_match_results(
//     conn: &PgConnection,
//     team_results: Vec<NewTeamMatchResult>,
// ) -> Result<Vec<TeamMatchResult>, diesel::result::Error> {
//     use crate::schema::team_match_results::{dsl, table};
//     diesel::insert_into(table)
//         .values(&team_results)
//         // TODO  confirm this on conflict actually works (have i set a unique const?)
//         .on_conflict((dsl::team_id, dsl::match_id))
//         .do_update()
//         .set((
//             dsl::meta.eq(excluded(dsl::meta)),
//             dsl::result.eq(excluded(dsl::result)),
//         ))
//         .get_results(conn)
// }

// pub fn upsert_player_match_results(
//     conn: &PgConnection,
//     player_results: Vec<NewPlayerResult>,
// ) -> Result<Vec<PlayerResult>, diesel::result::Error> {
//     use crate::schema::player_results::{dsl, table};
//     diesel::insert_into(table)
//         .values(&player_results)
//         // TODO  confirm this on conflict actually works (have i set a unique const?)
//         .on_conflict((dsl::player_id, dsl::match_id))
//         .do_update()
//         .set((
//             dsl::meta.eq(excluded(dsl::meta)),
//             dsl::result.eq(excluded(dsl::result)),
//         ))
//         .get_results(conn)
// }

// pub fn upsert_team_series_results(
//     conn: &PgConnection,
//     team_results: Vec<NewTeamSeriesResult>,
// ) -> Result<Vec<TeamSeriesResult>, diesel::result::Error> {
//     use crate::schema::team_series_results::{dsl, table};
//     diesel::insert_into(table)
//         .values(&team_results)
//         // TODO  confirm this on conflict actually works (have i set a unique const?)
//         .on_conflict((dsl::team_id, dsl::series_id))
//         .do_update()
//         .set((
//             dsl::meta.eq(excluded(dsl::meta)),
//             dsl::result.eq(excluded(dsl::result)),
//         ))
//         .get_results(conn)
// }

pub fn get_competition_ids_for_series(
    conn: &PgConnection,
    series_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
    use crate::schema::competitions;
    use crate::schema::series::dsl;

    dsl::series
        .select((dsl::series_id, dsl::competition_id))
        .filter(dsl::series_id.eq(any(series_ids)))
        .left_join(competitions::table)
        .load(conn)
}

pub fn get_competition_ids_for_matches(
    conn: &PgConnection,
    match_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
    use crate::schema::matches;
    use crate::schema::series;
    // matches::table.inner_join(series::table).load(conn)
    // TODO https://github.com/diesel-rs/diesel/issues/1129#issuecomment-324965108
    matches::table
        .inner_join(series::table)
        .select((matches::match_id, series::competition_id))
        .filter(matches::dsl::match_id.eq(any(match_ids)))
        .load(conn)
}

pub fn get_all_teams(conn: &PgConnection) -> Result<Vec<(Team, TeamName)>, diesel::result::Error> {
    use crate::schema::{team_names, teams};
    teams::table.inner_join(team_names::table).load(conn)
}

pub fn get_all_players(
    conn: &PgConnection,
) -> Result<Vec<(Player, PlayerName)>, diesel::result::Error> {
    use crate::schema::{player_names, players};
    players::table.inner_join(player_names::table).load(conn)
}

pub fn get_all_team_players(conn: &PgConnection) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
    use crate::schema::team_players;
    team_players::table.load(conn)
}

pub fn get_all_competitions(
    conn: &PgConnection,
) -> Result<Vec<Competition>, diesel::result::Error> {
    use crate::schema::competitions;
    competitions::table.load(conn)
}

pub fn get_full_competitions(
    conn: &PgConnection,
    competition_ids: Vec<Uuid>,
) -> Result<CompetitionHierarchy, diesel::result::Error> {
    use crate::schema::competitions;
    let comps = competitions::table
        .filter(competitions::dsl::competition_id.eq(any(competition_ids)))
        .load::<Competition>(conn)?;
    let series = Series::belonging_to(&comps).load::<Series>(conn)?;
    let matches = Match::belonging_to(&series).load::<Match>(conn)?;
    let team_series_results =
        TeamSeriesResult::belonging_to(&series).load::<TeamSeriesResult>(conn)?;
    let team_match_results =
        TeamMatchResult::belonging_to(&matches).load::<TeamMatchResult>(conn)?;
    let player_results = PlayerResult::belonging_to(&matches).load::<PlayerResult>(conn)?;
    let grouped_player_results = player_results.grouped_by(&matches);
    let grouped_team_match_results = team_match_results.grouped_by(&matches);
    let grouped_team_series_results = team_series_results.grouped_by(&series);
    let matches_and_match_results: Vec<Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>> =
        izip!(matches, grouped_player_results, grouped_team_match_results).grouped_by(&series);
    let series_lvl = izip!(
        series,
        grouped_team_series_results,
        matches_and_match_results
    )
    .grouped_by(&comps);
    let everything: CompetitionHierarchy = comps.into_iter().zip(series_lvl).collect();
    Ok(everything)
}
