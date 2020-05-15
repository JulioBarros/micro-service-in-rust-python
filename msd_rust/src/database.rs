/// Boilerplate code for connecting to the database and creating a connection pool
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use r2d2::Pool;

use std::env;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_pool() -> DBPool {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool");

    pool
}
