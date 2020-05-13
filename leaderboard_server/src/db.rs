use crate::schema;
use crate::types::leaderboards::*;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel_utils::PgConn;
use uuid::Uuid;

// TODO diesel lets you filter quite elegantly I think. Should probably look it up and use it
pub fn get_full_leagues(
    conn: &PgConn,
    league_id_filter: Option<Vec<&Uuid>>,
    leaderboard_id_filter: Option<Vec<&Uuid>>,
) -> Result<Vec<ApiLeaderboard>, diesel::result::Error> {
    let leaderboards: Vec<Leaderboard> = match (league_id_filter, leaderboard_id_filter) {
        (None, None) => schema::leaderboards::table.load(conn),
        (Some(league_ids), None) => schema::leaderboards::table
            .filter(schema::leaderboards::league_id.eq(any(league_ids)))
            .load(conn),
        (None, Some(leaderboard_ids)) => schema::leaderboards::table
            .filter(schema::leaderboards::leaderboard_id.eq(any(leaderboard_ids)))
            .load(conn),
        _ => panic!("cant be bothered."),
    }?;
    let players: Vec<Stat> = Stat::belonging_to(&leaderboards)
        .order(schema::stats::points.desc())
        .load(conn)?;
    let grouped_players = players.grouped_by(&leaderboards);
    let all: Vec<(Leaderboard, Vec<Stat>)> =
        leaderboards.into_iter().zip(grouped_players).collect();
    Ok(all
        .into_iter()
        .map(|rows| ApiLeaderboard::from(rows))
        .collect())
}

pub fn get_league_ids_to_leaderboard_ids(
    conn: &PgConn,
    leaderboard_ids: Vec<Uuid>,
) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
    schema::leaderboards::table
        .select((
            schema::leaderboards::leaderboard_id,
            schema::leaderboards::league_id,
        ))
        .filter(schema::leaderboards::leaderboard_id.eq(any(leaderboard_ids)))
        .load(conn)
}
