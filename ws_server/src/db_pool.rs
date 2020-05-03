use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

//https://github.com/seanmonstar/warp/issues/42#issuecomment-412265288
pub fn pg_pool(db_url: String) -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(&db_url);
    Pool::new(manager).expect(&format!("Could not connect to postgresql: {}", db_url))
}