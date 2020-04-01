use dotenv::dotenv;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use std::env;

pub type PgPool = Pool<ConnectionManager<PgConnection>>;
pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

//https://github.com/seanmonstar/warp/issues/42#issuecomment-412265288
pub fn pg_pool() -> PgPool{
    dotenv().ok();
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL env var must be set");
    let manager = ConnectionManager::<PgConnection>::new(&db_url);
    Pool::new(manager).expect(&format!("Could not connect to postgresql: {}", db_url))
}
