use crate::models::*;
use diesel::pg::expression::dsl::any;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
//use warp_ws_server::utils::my_timespan_format::DieselTimespan;

pub fn upsert_leagues<'a>(
    conn: &PgConnection,
    new: Vec<NewLeague>,
) -> Result<Vec<League>, diesel::result::Error> {
    use crate::schema::leagues::{dsl::*, table};
    // This "semi-upsert" doesnt work in postgres because it checks the inserts for null-ness, before other things,
    // so never fails the conflict check and goes into update part.
    // For now just do full upserts. fuck it.
    // let upsert_sql = "INSERT INTO competitions(competition_id, meta, name, timespan) VALUES (($1, $2, $3, $4), ($5, $6, $7, $8))
    //     ON CONFLICT DO UPDATE SET meta = coalesce(excluded.meta, meta), name = coalesce(excluded.name, name), timespan = coalesce(excluded.timespan, timespan)
    // "
    diesel::insert_into(table)
        .values(new)
        .on_conflict(league_id)
        .do_update()
        .set((
            meta.eq(excluded(meta)),
            name.eq(excluded(name)),
            team_size.eq(excluded(team_size)),
            competition_id.eq(excluded(competition_id)),
            squad_size.eq(excluded(squad_size)),
            max_players_per_team.eq(excluded(max_players_per_team)),
            max_players_per_position.eq(excluded(max_players_per_position)),
        ))
        .get_results(conn)
}