use crate::types::{leagues::*, users::*};
use diesel::pg::expression::dsl::any;
use diesel::pg::upsert::excluded;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use diesel::{sql_query, sql_types};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;
use crate::schema::*;
use itertools::izip;
//use warp_ws_server::utils::my_timespan_format::DieselTimespan;

pub fn get_full_leagues(conn: &PgConnection, league_ids: Vec<Uuid>) -> Result<Vec<ApiLeague>, diesel::result::Error>{
    let leagues: Vec<League> = leagues::table.filter(leagues::dsl::league_id.eq(any(league_ids))).load::<League>(conn)?;
    let periods = Period::belonging_to(&leagues).load::<Period>(conn)?;
    let stats = StatMultiplier::belonging_to(&leagues).load::<StatMultiplier>(conn)?;
    let grouped_periods = periods.grouped_by(&leagues);
    let grouped_stats = stats.grouped_by(&leagues);
    Ok(ApiLeague::from_rows(izip!(leagues, grouped_periods, grouped_stats).collect()))
}