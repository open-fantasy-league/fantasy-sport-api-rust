pub mod db_pool;
pub mod my_timespan_format;
pub mod my_timespan_format_opt;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::collections::Bound::{self, Excluded, Included, Unbounded};
use chrono::{DateTime, Utc};

pub use db_pool::pg_pool;
pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

pub type DieselTimespan = (Bound<DateTime<Utc>>, Bound<DateTime<Utc>>);

pub fn new_dieseltimespan(start: DateTime<Utc>, end: DateTime<Utc>) -> DieselTimespan{
    (Included(start), Excluded(end))
}

pub trait Timespan{
    fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self;
    fn lower(self) -> DateTime<Utc>;
    fn upper(self) -> DateTime<Utc>;
}

impl Timespan for DieselTimespan{
    fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self{
        (Included(start), Excluded(end))
    }

    fn lower(self) -> DateTime<Utc>{
        match self.0{
            Included(x) => x,
            Excluded(x) => x,
            Unbounded => panic!("Why the flying fudge is there an unbounded timestamp IN MY GOD DAMN DATABASE!!")
        }
    }

    fn upper(self) -> DateTime<Utc>{
        match self.1{
            Included(x) => x,
            Excluded(x) => x,
            Unbounded => panic!("Why the flying fudge is there an unbounded timestamp IN MY GOD DAMN DATABASE!!")
        }
    }
}

#[macro_export]
macro_rules! insert {
    ($conn:expr, $table:expr, $aggregate:expr) => {
        diesel::insert_into($table)
            .values($aggregate)
            .get_results($conn);
    };
}

#[macro_export]
macro_rules! insert_exec {
    ($conn:expr, $table:expr, $aggregate:expr) => {
        diesel::insert_into($table)
            .values($aggregate)
            .execute($conn);
    };
}

#[macro_export]
macro_rules! update {
    ($conn:expr, $table_name:ident, $pkey:ident, $changeset:expr) => {
        diesel::update(schema::$table_name::table).filter(schema::$table_name::dsl::$pkey.eq($changeset.$pkey))
            .set($changeset)
            .get_result($conn);
    };
}

#[macro_export]
macro_rules! update_2pkey {
    ($conn:expr, $table_name:ident, $pkey:ident, $pkey2:ident, $changeset:expr) => {
        diesel::update(schema::$table_name::table).filter(schema::$table_name::dsl::$pkey.eq($changeset.$pkey))
            .filter(schema::$table_name::dsl::$pkey2.eq($changeset.$pkey2))
            .set($changeset)
            .get_result($conn);
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
