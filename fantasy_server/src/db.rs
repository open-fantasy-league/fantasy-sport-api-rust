use crate::types::{leagues::*, users::*};
use diesel::pg::expression::dsl::any;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::schema::*;
use itertools::izip;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
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

pub fn get_full_leagues(conn: &PgConnection, league_ids: Vec<Uuid>) -> Result<Vec<ApiLeague>, diesel::result::Error>{
    let leagues: Vec<League> = leagues::table.filter(leagues::dsl::league_id.eq(any(league_ids))).load::<League>(conn)?;
    let periods = Period::belonging_to(&leagues).load::<Period>(conn)?;
    let stats = StatMultiplier::belonging_to(&leagues).load::<StatMultiplier>(conn)?;
    let grouped_periods = periods.grouped_by(&leagues);
    let grouped_stats = stats.grouped_by(&leagues);
    Ok(ApiLeague::from_rows(izip!(leagues, grouped_periods, grouped_stats).collect()))
}

pub fn get_users(conn: &PgConnection) -> Result<(Vec<ExternalUser>, Vec<Commissioner>), diesel::result::Error>{
    // TODO include what leagues user has team in
    let external_users = external_users::table.load::<ExternalUser>(conn)?;
    let commissioners = commissioners::table.load::<Commissioner>(conn)?;
    Ok((external_users, commissioners))
}

pub fn get_draft_ids_for_picks(conn: &PgConnection, pick_ids: &Vec<Uuid>,
) -> Result<Vec<(Uuid, Uuid)>, diesel::result::Error> {
    //DraftChoice, Draft

    picks::table.select((picks::dsl::pick_id, team_drafts::dsl::draft_id))
        .filter(picks::dsl::pick_id.eq(any(pick_ids)))
        .left_join(draft_choices::table)
        .left_join(team_drafts::table)
        .load(conn)
} 