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
    conn: PgConn,
    new: Vec<ApiTeamNameNew>,
) -> Result<Vec<TeamName>, diesel::result::Error> {
    use crate::schema::team_names;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let trimmed: Vec<_> = trim_timespans_team_name(&conn, "team_name", &new)?;
    //let trimmed: Vec<TeamName> = trim_timespans_many::<ApiTeamNameNew, TeamName>(conn, "team_name", new)?;
    let inserted: Vec<TeamName> = insert!(&conn, team_names::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

pub fn insert_player_names(
    conn: &PgConn,
    new: &Vec<ApiPlayerNameNew>,
) -> Result<Vec<PlayerName>, diesel::result::Error> {
    use crate::schema::player_names;
    // trim_timespans(conn, "team_name", t.team_id, new_timespan)
    let num_entries = new.len();
    let trimmed: Vec<_> = trim_timespans_player_name(conn, "player_name", new)?;
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
    let num_entries = new.len();
    let trimmed: Vec<_> = trim_timespans_player_position(conn, "player_position", new)?;
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
    let num_entries = new.len();
    let trimmed: Vec<_> = trim_timespans_team_player(conn, "team_player", new)?;
    let inserted: Vec<TeamPlayer> = insert!(conn, team_players::table, new)?;
    //inserted.append(&mut trimmed);
    Ok(inserted)
}

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

pub fn get_player_ids_to_team_ids(
    conn: &PgConnection,
    player_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
    use crate::schema::team_players;

    team_players::table
        .select((team_players::player_id, team_players::team_id))
        .filter(team_players::player_id.eq(any(player_ids)))
        .load(conn)
}

pub fn get_all_teams(conn: &PgConnection) -> Result<Vec<(Team, TeamName)>, diesel::result::Error> {
    use crate::schema::{team_names, teams};
    teams::table.inner_join(team_names::table).load(conn)
}

pub fn get_all_players(
    conn: &PgConnection,
) -> Result<Vec<(Player, PlayerName, PlayerPosition)>, diesel::result::Error> {
    use crate::schema::{player_names, players, player_positions};
    players::table.inner_join(player_names::table).inner_join(player_positions::table).load(conn)
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
    let everything: CompetitionHierarchy = comps.into_iter().zip(series_lvl).collect();
    Ok(everything)
}
