use actix_files::Files;
use actix_identity::IdentityMiddleware;
use actix_session::{config::PersistentSession, storage::CookieSessionStore, SessionMiddleware};
use actix_web::{
    cookie::{self, Key},
    web::{self},
    App, HttpServer,
};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use dotenv::dotenv;
use num_cpus;
use std::env;
use tokio_postgres::NoTls;

mod login;
use login::login_api;

mod get_groups;
use get_groups::get_groups_api;

mod get_group;
use get_group::get_group_api;

mod get_members;
use get_members::get_members_api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let db_host = env::var("DB_HOST").unwrap();
    let db_user = env::var("DB_USER").unwrap();
    let db_password = env::var("DB_PASSWORD").unwrap();
    let db_name = env::var("DB_NAME").unwrap();

    let num_workers = num_cpus::get();

    let mut pg_config = tokio_postgres::Config::new();
    pg_config.user(&db_user);
    pg_config.dbname(&db_name);
    pg_config.password(db_password);
    pg_config.host(&db_host);
    let mgr_config = ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    };
    let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
    let pool = Pool::builder(mgr).max_size(16).build().unwrap();

    let key = Key::generate();

    HttpServer::new(move || {
        App::new()
            .wrap(IdentityMiddleware::default())
            .wrap(
                SessionMiddleware::builder(CookieSessionStore::default(), key.clone())
                    .cookie_secure(true)
                    .cookie_http_only(true)
                    .cookie_same_site(cookie::SameSite::None)
                    .session_lifecycle(
                        PersistentSession::default().session_ttl(cookie::time::Duration::hours(2)),
                    )
                    .build(),
            )
            .app_data(web::Data::new(pool.clone()))
            .service(Files::new("/static", "./static/"))
            .service(login_api)
            .service(get_groups_api)
            .service(get_group_api)
            .service(get_members_api)
    })
    .workers(num_workers)
    .bind(("127.0.0.1", 9778))?
    .run()
    .await
}
