use crate::schema::{self, *};
use crate::types::{drafts::*, fantasy_teams::*, leagues::*, users::*, valid_players::*};
use diesel::pg::expression::dsl::any;
use diesel::pg::upsert::excluded;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use diesel_utils::PgConn;
use itertools::izip;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
// use diesel_utils::PgConn;
// use warp_ws_server::WSReq;
// use warp_ws_server::BoxError;
//use warp_ws_server::utils::my_timespan_format::DieselTimespan;

// use diesel::{
//     query_dsl::LoadQuery,
//     PgConnection,
//     Insertable,
//     query_builder::InsertStatement,
// };

// //https://www.reddit.com/r/rust/comments/afkuko/porting_go_to_rust_how_to_implement_a_generic/ee2jbfu?utm_source=share&utm_medium=web2x
// pub fn insert<Model, Table, Values>(req: WSReq<'_>, conn: PgConn, table: Table) -> Result<Vec<Model>, BoxError>
// where
//     Model: Serialize + DeserializeOwned,
//     Vec<Model>: Insertable<Table, Values=Values>,
//     InsertStatement<Table, Values>: LoadQuery<PgConnection, Model>
// {
//     let deserialized: Vec<Model> = serde_json::from_value(req.data)?;
//     Ok(diesel::insert_into(table).values(deserialized).get_results(&conn)?)
// }

pub fn get_full_leagues(
    conn: &PgConnection,
    league_ids_filter: Option<Vec<&Uuid>>,
) -> Result<Vec<ApiLeague>, diesel::result::Error> {
    let leagues: Vec<League> = match league_ids_filter {
        Some(league_ids) => leagues::table
            .filter(leagues::dsl::league_id.eq(any(league_ids)))
            .load::<League>(conn),
        None => leagues::table.load(conn),
    }?;
    let periods = Period::belonging_to(&leagues).load::<Period>(conn)?;
    let stats = StatMultiplier::belonging_to(&leagues).load::<StatMultiplier>(conn)?;
    let grouped_periods = periods.grouped_by(&leagues);
    let grouped_stats = stats.grouped_by(&leagues);
    Ok(ApiLeague::from_rows(
        izip!(leagues, grouped_periods, grouped_stats).collect(),
    ))
}

/*
#[derive(Serialize, Debug)]
pub struct ApiDraft {
    pub league_id: Uuid,
    pub draft_id: Uuid,
    pub period_id: Uuid,
    pub meta: serde_json::Value,
    pub choices: Vec<ApiDraftChoice>, //pub teams: Vec<ApiTeamDraft>,
}
*/

pub fn get_full_drafts(
    conn: &PgConnection,
    league_ids_filter: Option<Vec<&Uuid>>,
) -> Result<Vec<ApiLeague>, diesel::result::Error> {
    let leagues: Vec<League> = match league_ids_filter {
        Some(league_ids) => leagues::table
            .filter(leagues::dsl::league_id.eq(any(league_ids)))
            .load::<League>(conn),
        None => leagues::table.load(conn),
    }?;
    let periods = Period::belonging_to(&leagues).load::<Period>(conn)?;
    let stats = StatMultiplier::belonging_to(&leagues).load::<StatMultiplier>(conn)?;
    let grouped_periods = periods.grouped_by(&leagues);
    let grouped_stats = stats.grouped_by(&leagues);
    Ok(ApiLeague::from_rows(
        izip!(leagues, grouped_periods, grouped_stats).collect(),
    ))
}

pub fn get_users(
    conn: &PgConnection,
) -> Result<(Vec<ExternalUser>, Vec<Commissioner>), diesel::result::Error> {
    // TODO include what leagues user has team in
    let external_users = external_users::table.load::<ExternalUser>(conn)?;
    let commissioners = commissioners::table.load::<Commissioner>(conn)?;
    Ok((external_users, commissioners))
}

pub fn get_draft_ids_for_picks(
    conn: &PgConnection,
    pick_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
    picks::table
        // important to inner_join between draft-choices and team-drafts (cant do innerjoin().innerjoin(), as that tries joining picks)
        .inner_join(draft_choices::table.inner_join(team_drafts::table))
        .select((picks::pick_id, team_drafts::draft_id))
        .filter(picks::dsl::pick_id.eq(any(pick_ids)))
        .load(conn)
}

pub fn get_undrafted_periods(conn: PgConn) -> Result<Vec<Period>, diesel::result::Error> {
    periods::table
        .select(periods::all_columns)
        .left_join(drafts::table)
        .filter(drafts::draft_id.is_null())
        .order(periods::draft_lockdown)
        .load::<Period>(&conn)
    //.first::<Period>(conn)
}

pub fn get_valid_picks(
    conn: &PgConnection,
    period_id: Uuid,
) -> Result<Vec<Uuid>, diesel::result::Error> {
    valid_players::table
        .select(valid_players::player_id)
        .filter(valid_players::period_id.eq(period_id))
        .load(conn)
}

pub fn get_unchosen_draft_choices(
    conn: PgConn,
) -> Result<Vec<(DraftChoice, Period, TeamDraft, League)>, diesel::result::Error> {
    // So this would join every row, including old rows, then filter most of them out.
    // Should check postgresql optimises nicely.

    // TODO this is way too heavyweight for being called every draft-choice
    // really once draft is fixed, the max_per_blah settings shouldnt be changing. Same for period timespan.
    // When fantasy-teams/users are locked in for draft, then settings should lock as well, and be pulled into memory

    draft_choices::table
        .left_join(picks::table)
        .filter(picks::pick_id.is_null())
        .inner_join(
            team_drafts::table
                .inner_join(drafts::table.inner_join(periods::table.inner_join(leagues::table))),
        )
        .select((
            draft_choices::all_columns,
            periods::all_columns,
            team_drafts::all_columns,
            leagues::all_columns,
        ))
        .load(&conn)
}

pub fn get_randomised_teams_for_league(
    conn: &PgConnection,
    league_id: Uuid,
) -> Result<Vec<FantasyTeam>, diesel::result::Error> {
    // Whilst order by random is expensive on huge tables, I think will only have small amount teams per league so should be fine. finger-cross
    no_arg_sql_function!(
        random,
        sql_types::Integer,
        "Represents the SQL RANDOM() function"
    );

    fantasy_teams::table
        .filter(schema::fantasy_teams::league_id.eq(league_id))
        .order(random)
        .load(conn)
}

pub fn get_league_squad_size(
    conn: &PgConnection,
    league_id: Uuid,
) -> Result<i32, diesel::result::Error> {
    schema::leagues::table
        .select(schema::leagues::squad_size)
        .filter(schema::leagues::league_id.eq(league_id))
        .get_result(conn)
}

pub fn get_draft_queue_for_choice(
    conn: &PgConnection,
    unchosen: DraftChoice,
) -> Result<Vec<Uuid>, diesel::result::Error> {
    // maybe no queue been upserted. could be empty, could be missing?
    schema::team_drafts::table
        .inner_join(schema::fantasy_teams::table.inner_join(schema::draft_queues::table))
        .inner_join(schema::draft_choices::table)
        .filter(schema::team_drafts::team_draft_id.eq(unchosen.team_draft_id))
        .select(schema::draft_queues::player_ids)
        .get_result(conn)
}

pub fn get_current_picks(
    conn: &PgConnection,
    fantasy_team_id: Uuid,
    period_id: Uuid,
) -> Result<Vec<Uuid>, diesel::result::Error> {
    picks::table
        .select(picks::pick_id)
        .filter(picks::fantasy_team_id.eq(fantasy_team_id))
        .inner_join(draft_choices::table.inner_join(team_drafts::table.inner_join(drafts::table)))
        .filter(drafts::period_id.eq(period_id))
        .load(conn)
}

pub fn upsert_active_picks(
    conn: &PgConnection,
    data: &Vec<ActivePick>,
) -> Result<Vec<ActivePick>, diesel::result::Error> {
    diesel::insert_into(active_picks::table)
        .values(data)
        // constraint unique pick-with timespan (want same on pick itself so only 1 of player in squad)
        .on_conflict((active_picks::pick_id, active_picks::timespan))
        .do_update()
        .set(active_picks::timespan.eq(excluded(active_picks::timespan)))
        //.set(name.eq(coalesce::<sql_types::Text>(excluded(name), name)))
        .get_results(conn)
}

#[derive(QueryableByName)]
pub struct VecUuid {
    #[sql_type = "sql_types::Array<sql_types::Uuid>"]
    pub inner: Vec<Uuid>,
}

pub fn get_all_updated_teams(
    conn: &PgConnection,
    ids: Vec<Uuid>,
) -> Result<Vec<VecUuid>, diesel::result::Error> {
    // https://www.reddit.com/r/PostgreSQL/comments/gjsham/query_to_list_combinations_of_band_members/
    let sql = "
        WITH upper_and_lower AS (select t1.timespan,
        (select array_agg(t2.pick_id) 
         from active_picks t2
         where t2.timespan @> lower(t1.timespan)) as lower_ids,
        (select array_agg(t3.pick_id) 
         from active_picks t3
         where t3.timespan @> upper(t1.timespan)) as upper_ids
        from (
            select timespan
            from active_picks where pick_id = ANY($1)
        ) as t1)

        select distinct ids from 
        (select lower_ids as ids, lower(timespan) as ttime from upper_and_lower 
        union 
            select upper_ids as ids, upper(timespan) as ttime from upper_and_lower
        ) as final_sub;
    ";
    sql_query(sql)
        .bind::<sql_types::Array<sql_types::Uuid>, _>(ids)
        .load::<VecUuid>(conn)
}

pub fn get_leagues_for_picks(
    conn: &PgConnection,
    pick_ids: Vec<Uuid>,
) -> Result<Vec<League>, diesel::result::Error> {
    picks::table
        .inner_join(fantasy_teams::table.inner_join(leagues::table))
        .select(leagues::all_columns)
        .load(conn)
}
