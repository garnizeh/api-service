mod router;
mod api;
mod todo;
mod error;

#[tokio::main]
async fn main() {
    init_tracing();

    let dbpool = init_dbpool().await.expect("couldn't initialize DB pool");

    let router = router::create_router(dbpool).await;

    let bind_addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:3000".to_string());

    // run our app with hyper
    let listener = tokio::net::TcpListener::bind(bind_addr)
        .await
        .expect("unable to listen tcp addr");

    axum::serve(listener, router.into_make_service())
        .await
        .expect("unable to start server");
}

fn init_tracing() {
    use tracing_subscriber::{filter::LevelFilter, fmt, prelude::*, EnvFilter};

    let rust_log = std::env::var(EnvFilter::DEFAULT_ENV)
        .unwrap_or_else(|_| "sqlx=info,tower_http=debug,info".to_string());

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .parse_lossy(rust_log),
        )
        .init();
}

async fn init_dbpool() -> Result<sqlx::Pool<sqlx::Sqlite>, sqlx::Error> {
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;

    let db_connection_str =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:db.sqlite".to_string());

    let dbpool = SqlitePoolOptions::new()
        .connect_with(SqliteConnectOptions::from_str(&db_connection_str)?.create_if_missing(true))
        .await
        .expect("can't connect to database");

    sqlx::migrate!()
        .run(&dbpool)
        .await
        .expect("database migration failed");

    Ok(dbpool)
}
