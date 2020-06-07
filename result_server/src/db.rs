#![macro_use]
use diesel::pg::expression::dsl::any;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use uuid::Uuid;
use itertools::{izip, Itertools};
use diesel_utils::{PgConn};
use crate::types::{competitions::*, series::*, matches::*, teams::*, results::*, players::*};
use crate::schema;
use std::collections::HashMap;
use frunk::labelled::transform_from;

//sql_function! {fn coalesce<T: sql_types::NotNull>(a: sql_types::Nullable<T>, b: T) -> T;}
//sql_function!(fn trim_team_name_timespans(new_team_id sql_types::Uuid, new_timespan sql_types::Range<sql_types::Timestamptz>) -> ());
//sql_function!(trim_team_name_timespans, WTF, (new_team_id: sql_types::Uuid, new_timespan: sql_types::Range<sql_types::Timestamptz>) -> TeamName);

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
pub fn insert_team_names(
    conn: &PgConn,
    new: Vec<ApiTeamNameNew>,
) -> Result<Vec<TeamName>, diesel::result::Error> {
    use crate::schema::team_names;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let _: Vec<_> = trim_timespans_team_name(conn, "team_name", &new)?;
    //let trimmed: Vec<TeamName> = trim_timespans_many::<ApiTeamNameNew, TeamName>(conn, "team_name", new)?;
    let inserted: Vec<TeamName> = insert!(conn, team_names::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub fn insert_player_names(
    conn: &PgConn,
    new: &Vec<ApiPlayerNameNew>,
) -> Result<Vec<PlayerName>, diesel::result::Error> {
    use crate::schema::player_names;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let _: Vec<_> = trim_timespans_player_name(conn, "player_name", new)?;
    let inserted: Vec<PlayerName> = insert!(conn, player_names::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub fn insert_player_positions(
    conn: &PgConn,
    new: &Vec<ApiPlayerPositionNew>,
) -> Result<Vec<PlayerPosition>, diesel::result::Error> {
    use crate::schema::player_positions;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let _: Vec<_> = trim_timespans_player_position(conn, "player_position", new)?;
    let inserted: Vec<PlayerPosition> = insert!(conn, player_positions::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub fn insert_team_players(
    conn: &PgConn,
    new: &Vec<ApiTeamPlayer>,
) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
    use crate::schema::team_players;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let _: Vec<_> = trim_timespans_team_player(conn, "team_player", new)?;
    let inserted: Vec<TeamPlayer> = insert!(conn, team_players::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

// pub fn get_competition_ids_for_series(
//     conn: &PgConnection,
//     series_ids: &Vec<Uuid>,
// ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
//     use crate::schema::competitions;
//     use crate::schema::series::dsl;

//     dsl::series
//         .select((dsl::series_id, dsl::competition_id))
//         .filter(dsl::series_id.eq(any(series_ids)))
//         .left_join(competitions::table)
//         .load(conn)
// }

// pub fn get_competition_ids_for_matches(
//     conn: &PgConnection,
//     match_ids: &Vec<Uuid>,
// ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
//     use crate::schema::matches;
//     use crate::schema::series;
//     // matches::table.inner_join(series::table).load(conn)
//     // TODO https://github.com/diesel-rs/diesel/issues/1129#issuecomment-324965108
//     matches::table
//         .inner_join(series::table)
//         .select((matches::match_id, series::competition_id))
//         .filter(matches::dsl::match_id.eq(any(match_ids)))
//         .load(conn)
// }

// pub fn get_player_ids_to_team_ids(
//     conn: &PgConnection,
//     player_ids: &Vec<Uuid>,
// ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
//     use crate::schema::team_players;

//     team_players::table
//         .select((team_players::player_id, team_players::team_id))
//         .filter(team_players::player_id.eq(any(player_ids)))
//         .load(conn)
// }

// pub fn get_all_teams(conn: &PgConnection) -> Result<Vec<(Team, TeamName)>, diesel::result::Error> {
//     use crate::schema::{team_names, teams};
//     teams::table.inner_join(team_names::table).load(conn)
// }

// pub fn get_all_players(
//     conn: &PgConnection,
// ) -> Result<Vec<(Player, PlayerName, PlayerPosition)>, diesel::result::Error> {
//     use crate::schema::{player_names, players, player_positions};
//     players::table.inner_join(player_names::table).inner_join(player_positions::table).load(conn)
// }

// pub fn get_all_team_players(conn: &PgConnection) -> Result<Vec<TeamPlayer>, diesel::result::Error> {
//     use crate::schema::team_players;
//     team_players::table.load(conn)
// }

// pub fn get_all_competitions(
//     conn: &PgConnection,
// ) -> Result<Vec<Competition>, diesel::result::Error> {
//     use crate::schema::competitions;
//     competitions::table.load(conn)
// }

pub fn get_full_competitions(
    conn: &PgConnection,
    competition_ids_filter: Option<Vec<&Uuid>>,
) -> Result<CompetitionHierarchy, diesel::result::Error> {
    use crate::schema::competitions;
    let comps = match competition_ids_filter{
        Some(competition_ids) => competitions::table
        .filter(competitions::dsl::competition_id.eq(any(competition_ids)))
        .load::<Competition>(conn),
        None => competitions::table.load::<Competition>(conn)
    }?;
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
    let everything: CompetitionHierarchy = comps.into_iter().zip(series_lvl.into_iter().map(Some).collect_vec()).collect();
    Ok(everything)
}


pub fn get_publishable_matches(conn: &PgConnection, data: Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>) -> Result<Vec<CompetitionHierarchyMatchRow>, diesel::result::Error>{
    let inserted_ids: Vec<Uuid> = data.iter().map(|x| x.0.series_id).collect();
    let series = schema::series::table.filter(schema::series::series_id.eq(any(inserted_ids))).load::<Series>(conn)?;
    let comp_ids: Vec<Uuid> = series.iter().map(|s| s.competition_id).collect();
    let comps = schema::competitions::table.filter(schema::competitions::competition_id.eq(any(comp_ids))).load::<Competition>(conn)?;
    let grouped_matches = data.grouped_by(&series);
    let series_level: Vec<(Series,Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>)> = series.into_iter().zip(grouped_matches).collect();
    let grouped_comps = series_level.grouped_by(&comps);
    Ok(comps.into_iter().zip(grouped_comps).collect())
}

pub fn get_publishable_series(
    conn: &PgConnection, data: Vec<(Series, Vec<TeamSeriesResult>, Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>)>
) -> Result<CompetitionHierarchy, diesel::result::Error>{
    let comp_ids: Vec<Uuid> = data.iter().map(|x| x.0.competition_id).collect();
    let comps = schema::competitions::table.filter(schema::competitions::competition_id.eq(any(comp_ids))).load::<Competition>(conn)?;
    let grouped = data.grouped_by(&comps);
    Ok(comps.into_iter().zip(grouped.into_iter().map(Some).collect_vec()).collect())
}

pub fn get_publishable_team_series_results(conn: &PgConnection, data: Vec<TeamSeriesResult>) -> Result<CompetitionHierarchy, diesel::result::Error>{
    let inserted_ids: Vec<Uuid> = data.iter().map(|x| x.series_id).collect();
    let series = schema::series::table.filter(schema::series::series_id.eq(any(inserted_ids))).load::<Series>(conn)?;
    let comp_ids: Vec<Uuid> = series.iter().map(|s| s.competition_id).collect();
    let comps = schema::competitions::table.filter(schema::competitions::competition_id.eq(any(comp_ids))).load::<Competition>(conn)?;
    let grouped_by_series = data.grouped_by(&series);
    let series_level: Vec<(Series, Vec<TeamSeriesResult>, Vec<(Match, Vec<PlayerResult>, Vec<TeamMatchResult>)>)> = izip!(
        series, grouped_by_series, None
    ).collect();
    let grouped_comps = series_level.grouped_by(&comps).into_iter().map(Some).collect_vec();
    Ok(comps.into_iter().zip(grouped_comps).collect())
}

// TODO commonise this stuff
pub fn get_publishable_team_match_results(conn: &PgConnection, data: Vec<TeamMatchResult>) -> Result<Vec<CompetitionHierarchyOptyRow>, diesel::result::Error>{
    let inserted_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    let matches = schema::matches::table.filter(schema::matches::match_id.eq(any(inserted_ids))).load::<Match>(conn)?;
    let series_ids: Vec<Uuid> = matches.iter().map(|s| s.series_id).collect();
    let series = schema::series::table.filter(schema::series::series_id.eq(any(series_ids))).load::<Series>(conn)?;
    let comp_ids: Vec<Uuid> = series.iter().map(|s| s.competition_id).collect();
    let comps = schema::competitions::table.filter(schema::competitions::competition_id.eq(any(comp_ids))).load::<Competition>(conn)?;

    let grouped_by_matches = data.grouped_by(&matches);
    let match_len = matches.len();
    let match_stuff: Vec<(Match, Option<Vec<PlayerResult>>, Option<Vec<TeamMatchResult>>)> = izip!(
        matches, vec![None; match_len], grouped_by_matches.into_iter().map(Some).collect_vec()
    ).collect();
    let grouped_by_series = match_stuff.grouped_by(&series);
    let series_len = series.len();
    let series_level: Vec<(Series, Option<Vec<TeamSeriesResult>>, Option<Vec<(Match, Option<Vec<PlayerResult>>, Option<Vec<TeamMatchResult>>)>>)> = izip!(
        series, vec![None; series_len], grouped_by_series.into_iter().map(Some).collect_vec()
    ).collect();
    let grouped_comps = series_level.grouped_by(&comps).into_iter().map(Some).collect_vec();
    Ok(comps.into_iter().zip(grouped_comps).collect())
}

pub fn get_publishable_player_results(conn: &PgConnection, data: Vec<PlayerResult>) -> Result<Vec<CompetitionHierarchyOptyRow>, diesel::result::Error>{
    let inserted_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
    let matches = schema::matches::table.filter(schema::matches::match_id.eq(any(inserted_ids))).load::<Match>(conn)?;
    let series_ids: Vec<Uuid> = matches.iter().map(|s| s.series_id).collect();
    let series = schema::series::table.filter(schema::series::series_id.eq(any(series_ids))).load::<Series>(conn)?;
    let comp_ids: Vec<Uuid> = series.iter().map(|s| s.competition_id).collect();
    let comps = schema::competitions::table.filter(schema::competitions::competition_id.eq(any(comp_ids))).load::<Competition>(conn)?;

    let grouped_by_matches = data.grouped_by(&matches);
    let matches_len = matches.len();
    let match_stuff: Vec<(Match, Option<Vec<PlayerResult>>, Option<Vec<TeamMatchResult>>)> = izip!(
        matches, grouped_by_matches.into_iter().map(Some).collect_vec(), vec![None; matches_len]
    ).collect();
    let grouped_by_series = match_stuff.grouped_by(&series);
    let series_len = series.len();
    let series_level: Vec<(Series, Option<Vec<TeamSeriesResult>>, Option<Vec<(Match, Option<Vec<PlayerResult>>, Option<Vec<TeamMatchResult>>)>>)> = izip!(
        series, vec![None; series_len], grouped_by_series.into_iter().map(Some).collect_vec()
    ).collect();
    let grouped_comps = series_level.grouped_by(&comps).into_iter().map(Some).collect_vec();
    Ok(comps.into_iter().zip(grouped_comps).collect())
}

// pub fn get_hierarchy_for_team_ids(conn: &PgConnection, team_ids: Vec<Uuid>) -> Result<Vec<CompetitionHierarchyOptyRow>, diesel::result::Error>{
//     let inserted_ids: Vec<Uuid> = data.iter().map(|x| x.match_id).collect();
//     let matches = schema::matches::table.filter(schema::matches::match_id.eq(any(inserted_ids))).load::<Match>(conn)?;
//     let series_ids: Vec<Uuid> = matches.iter().map(|s| s.series_id).collect();
//     let series = schema::series::table.filter(schema::series::series_id.eq(any(series_ids))).load::<Series>(conn)?;
//     let comp_ids: Vec<Uuid> = series.iter().map(|s| s.competition_id).collect();
//     let comps = schema::competitions::table.filter(schema::competitions::competition_id.eq(any(comp_ids))).load::<Competition>(conn)?;

//     let grouped_by_matches = data.grouped_by(&matches);
//     let matches_len = matches.len();
//     let match_stuff: Vec<(Match, Option<Vec<PlayerResult>>, Option<Vec<TeamMatchResult>>)> = izip!(
//         matches, grouped_by_matches.into_iter().map(Some).collect_vec(), vec![None; matches_len]
//     ).collect();
//     let grouped_by_series = match_stuff.grouped_by(&series);
//     let series_len = series.len();
//     let series_level: Vec<(Series, Option<Vec<TeamSeriesResult>>, Option<Vec<(Match, Option<Vec<PlayerResult>>, Option<Vec<TeamMatchResult>>)>>)> = izip!(
//         series, vec![None; series_len], grouped_by_series.into_iter().map(Some).collect_vec()
//     ).collect();
//     let grouped_comps = series_level.grouped_by(&comps);
//     Ok(comps.into_iter().zip(grouped_comps).collect())
// }

pub fn get_teams_from_players(conn: &PgConn, player_ids_filt: Option<Vec<Uuid>>)-> Result<Vec<ApiTeamWithPlayersHierarchy>, diesel::result::Error>{
    println!("In get_teams_from_players");
    // let rows: Vec<(Player, TeamPlayer, Team, PlayerName, PlayerPosition)> = schema::players::table.filter(schema::players::player_id.eq(any(player_ids)))
    //    .inner_join(schema::team_players::table.inner_join(schema::teams::table))
    //    .inner_join(schema::player_names::table)
    //    .inner_join(schema::player_positions::table)
    //    .load(conn)?;
    let team_players: Vec<TeamPlayer> = match player_ids_filt{
        Some(ref player_ids) => schema::team_players::table.filter(schema::team_players::player_id.eq(any(&player_ids))).load(conn),
        None => schema::team_players::table.load(conn)
    }?;
    //let players: Vec<Player> = Player::belonging_to(&team_players).load::<Player>(conn)?;
    let players: Vec<Player> = match player_ids_filt{
        Some(player_ids) => schema::players::table.filter(schema::players::player_id.eq(any(&player_ids))).load(conn),
        None => schema::players::table.load(conn)
    }?;
    //let team_players: Vec<TeamPlayer> = TeamPlayer::belonging_to(&players).load::<TeamPlayer>(conn)?;
    let player_names = PlayerName::belonging_to(&players).load::<PlayerName>(conn)?;
    let player_positions = PlayerPosition::belonging_to(&players).load::<PlayerPosition>(conn)?;
    let grouped_player_names = player_names.grouped_by(&players);
    let grouped_player_positions = player_positions.grouped_by(&players);
    let grouped_players: Vec<(Player, Vec<PlayerName>, Vec<PlayerPosition>)> = izip!(players, grouped_player_names, grouped_player_positions).collect();
    let api_players = ApiPlayer::from_diesel_rows(grouped_players);
    let team_ids = team_players.iter().map(|tp| tp.team_id).dedup().collect_vec();
    let teams: Vec<Team> = schema::teams::table.filter(schema::teams::team_id.eq(any(&team_ids))).load(conn)?;
    let team_names: Vec<TeamName> = TeamName::belonging_to(&teams).load(conn)?;
    let nested_team_names = team_names.grouped_by(&teams);
    let grouped_team_names: Vec<(Team, Vec<TeamName>)> = teams.into_iter().zip(nested_team_names).collect_vec();
    //use diesel::debug_query;
    //use diesel::pg::Pg;
    //let q = schema::teams::table.filter(schema::teams::team_id.eq(any(&team_ids)));
    //let debug = debug_query::<Pg, _>(&q);
    //println!("{}", debug);
    let mut teams_to_team_players: HashMap<Uuid, Vec<TeamPlayer>> = team_players.into_iter().fold(HashMap::new(), |mut hm, tp| {
        match hm.get_mut(&tp.team_id) {
            Some(v) => {
                v.push(tp);
            }
            None => {
                hm.insert(tp.team_id, vec![tp]);
            }
        };
        hm
    });
    let mut player_map: HashMap<Uuid, ApiPlayer> = api_players.into_iter().map(|p| (p.player_id, p)).collect();
    //let team_players_grouped = team_players.grouped_by(&teams);
    let out = grouped_team_names.into_iter().map(|(t, team_names)|{
        let team_players = teams_to_team_players.remove(&t.team_id).unwrap_or(vec![]);
        let players = team_players.into_iter().map(|tp| {
            let api_player = player_map.remove(&tp.player_id).unwrap();
            ApiTeamPlayerOut{
                team_id: tp.team_id,
                timespan: tp.timespan,
                player: api_player
            }
        }).collect_vec();
        ApiTeamWithPlayersHierarchy{
            team_id: t.team_id,
            names: Some(team_names.into_iter().map(transform_from).collect_vec()),
            meta: t.meta,
            players: Some(players)
        }
    }).collect_vec();
    Ok(out)
}

pub fn get_teams_names(conn: &PgConn, team_ids: Vec<Uuid>)-> Result<Vec<ApiTeamWithPlayersHierarchy>, diesel::result::Error>{
    let teams: Vec<Team> = schema::teams::table.filter(schema::teams::team_id.eq(any(&team_ids))).load(conn)?;
    let team_names: Vec<TeamName> = TeamName::belonging_to(&teams).load(conn)?;
    let grouped_names = team_names.grouped_by(&teams);
    let out = teams.into_iter().zip(grouped_names).map(|(t, tnames)|{
        ApiTeamWithPlayersHierarchy{
            team_id: t.team_id,
            names: Some(tnames.into_iter().map(transform_from).collect_vec()),
            meta: t.meta,
            players: None
        }
    }).collect_vec();
    Ok(out)
}

/*
pub struct ApiTeamWithPlayersHierarchy{
    pub team_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub names: Option<Vec<ApiTeamName>>,
    pub meta: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub players: Option<Vec<ApiPlayer>>
}
*/