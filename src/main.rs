use clap::Parser;
use sea_orm::{Database, DatabaseConnection};

use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use backend::migration::{Migrator, MigratorTrait};
use tower_http::trace::TraceLayer;

mod handlers;

// Setup the command line interface with clap.
#[derive(Parser, Debug)]
#[clap(name = "api", about = "An API for our project!")]
struct Opt {
    /// set the log level
    #[clap(short = 'l', long = "log", default_value = "debug")]
    log_level: String,

    /// set the listen addr
    #[clap(short = 'a', long = "addr", default_value = "0.0.0.0")]
    addr: String,

    /// set the listen port
    #[clap(short = 'p', long = "port", default_value = "8000")]
    port: u16,
}

#[derive(Clone)]
struct AppState {
    conn: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    // Load the environment variables from the .env file.
    dotenvy::dotenv().ok();

    // Parse the command line arguments.
    let opt = Opt::parse();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");

    // Setup logging & RUST_LOG from args
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    // Enable console logging
    tracing_subscriber::fmt::init();

    // Create the database connection pool.
    // let pool =
    let conn = Database::connect(db_url)
        .await
        .expect("Database connection failed");
    Migrator::up(&conn, None).await.unwrap();

    let state = AppState { conn };

    //     let app = Router::new()
    //         .route("/api/posts", get(list_posts_by_page))
    //         .route("/api/posts/all", get(list_posts))
    //         .route("/api/post", post(new_post))
    //         .route("/api/post/:id", post(edit_post).delete(delete_post))
    //         .layer(CorsLayer::permissive().allow_origin(Any))
    //         .layer(TraceLayer::new_for_http())
    //         // .layer(CookieManagerLayer::new())
    //         .with_state(state);

    let router = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/api/posts", get(handlers::post::list_posts_by_page))
        .route("/api/posts/all", get(handlers::post::list_posts))
        .route("/api/post", post(handlers::post::new_post))
        .route(
            "/api/post/:id",
            post(handlers::post::edit_post).delete(handlers::post::delete_post),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start the server.
    tracing::info!("Listening on http://{}:{}", opt.addr, opt.port);

    let listener = tokio::net::TcpListener::bind(format!("{}:{}", opt.addr, opt.port))
        .await
        .expect("Failed to bind port");
    axum::serve(listener, router).await.unwrap();
}
