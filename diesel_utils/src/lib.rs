pub mod db_pool;
pub mod my_timespan_format;
pub mod my_timespan_format_opt;
pub use my_timespan_format::DieselTimespan;

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
