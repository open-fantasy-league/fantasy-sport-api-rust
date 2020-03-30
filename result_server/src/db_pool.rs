use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

//https://github.com/seanmonstar/warp/issues/42#issuecomment-412265288
pub fn pg_pool() -> PgPool{
    let manager = ConnectionManager::<PgConnection>::new("postgres://fantasy:fantasy@localhost/fantasy_results");
    Pool::new(manager).expect("Postgresql connection pool could not be created")
}
