use crate::schema;
use crate::types::*;
use diesel::associations::*;
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use diesel_utils::PgConn;
use itertools::Itertools;
use std::collections::HashMap;
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

// pub fn get_league_ids_to_leaderboard_ids(
//     conn: &PgConn,
//     leaderboard_ids: Vec<Uuid>,
// ) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
//     schema::leaderboards::table
//         .select((
//             schema::leaderboards::leaderboard_id,
//             schema::leaderboards::league_id,
//         ))
//         .filter(schema::leaderboards::leaderboard_id.eq(any(leaderboard_ids)))
//         .load(conn)
// }

// Would be great to do a join when inserting and can do a clever returning,
// However don't think diesel supports fancy enough behaviour.
pub fn get_stat_with_ids(
    conn: &PgConn,
    data: Vec<Stat>,
) -> Result<Vec<(Leaderboard, Vec<Stat>)>, diesel::result::Error> {
    let inserted_ids: Vec<Uuid> = data.iter().map(|x| x.leaderboard_id).collect();
    let leagues = schema::leaderboards::table
        .filter(schema::leaderboards::leaderboard_id.eq(any(inserted_ids)))
        .load::<Leaderboard>(conn)?;
    let grouped: Vec<Vec<Stat>> = data.grouped_by(&leagues);
    let out = leagues.into_iter().zip(grouped).collect();
    Ok(out)
}

pub fn latest_leaderboards(
    conn: &PgConn,
    leaderboard_ids: Vec<Uuid>,
) -> Result<Vec<ApiLeaderboardLatest>, diesel::result::Error> {
    let leaderboards: Vec<Leaderboard> = schema::leaderboards::table
        .filter(schema::leaderboards::leaderboard_id.eq(any(&leaderboard_ids)))
        .load(conn)?;
    let sql = "
        SELECT player_id, leaderboard_id, (MAX(ARRAY[EXTRACT('EPOCH' FROM timestamp)::float, points]))[2] AS points 
        FROM stats WHERE leaderboard_id = ANY($1) 
        GROUP BY player_id, leaderboard_id
        ORDER BY points
    ";
    let stats: Vec<ApiLatestStat> = sql_query(sql)
        .bind::<sql_types::Array<sql_types::Uuid>, _>(leaderboard_ids)
        .load::<ApiLatestStat>(conn)?;
    let mut grouped_stats: HashMap<Uuid, (Leaderboard, Vec<ApiLatestStat>)> = leaderboards
        .into_iter()
        .map(|x| (x.leaderboard_id, (x, vec![])))
        .collect();
    for stat in stats {
        grouped_stats
            .get_mut(&stat.leaderboard_id)
            .unwrap()
            .1
            .push(stat);
    }
    Ok(grouped_stats
        .into_iter()
        .map(|(_, (leaderboard, stats))| ApiLeaderboardLatest {
            leaderboard_id: leaderboard.leaderboard_id,
            league_id: leaderboard.league_id,
            name: leaderboard.name,
            meta: leaderboard.meta,
            leaderboard: stats,
        })
        .collect_vec())
}
