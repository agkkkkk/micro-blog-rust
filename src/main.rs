use actix_web::{middleware, web::Data, App, HttpServer};
use diesel::{r2d2::ConnectionManager, PgConnection};
use dotenv::dotenv;

use r2d2::{Pool, PooledConnection};
use std::{env, io::Result};

mod like;
mod microblog;
mod response;
mod schema;

pub type DBPool = Pool<ConnectionManager<PgConnection>>;
pub type DBPooledConnection = PooledConnection<ConnectionManager<PgConnection>>;

#[actix_rt::main]
async fn main() -> Result<()> {
    env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL");
    let db_conn = ConnectionManager::<PgConnection>::new(db_url);

    let pool = r2d2::Pool::builder()
        .build(db_conn)
        .expect("Failed to create pool");

    HttpServer::new(move || {
        App::new()
            .app_data(Data::new(pool.clone()))
            .wrap(middleware::Logger::default())
            .service(microblog::blogs)
            .service(microblog::create_blogs)
            .service(microblog::get_blog)
            .service(microblog::delete_blog)
            .service(like::list)
            .service(like::like_blog)
            .service(like::dislike_blog)
    })
    .bind("127.0.0.1:8000")?
    .run()
    .await
}
