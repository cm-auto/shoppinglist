use actix_web::{
    middleware::{self, Logger},
    web::{self},
    App, HttpServer, Scope,
};

struct AppData {
    pool: sqlx::PgPool,
}

// these two macros would also be used if there would be a "v2" of the api
// this is why they are in the main module
#[macro_export]
macro_rules! generate_options_response {
    ($allow_str: expr) => {
        || async {
            use actix_web::{http, HttpResponseBuilder};
            HttpResponseBuilder::new(http::StatusCode::NO_CONTENT)
                .insert_header((http::header::ALLOW, $allow_str))
                .finish()
        }
    };
}

#[macro_export]
macro_rules! generate_options_route {
    ($allow_str: expr) => {
        web::method(Method::OPTIONS).to(generate_options_response!($allow_str))
    };
}

mod auth;
mod v1;

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().expect("Failed to load .env file");

    let database_url = dotenvy::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pg_pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    let app_data = web::Data::new(AppData { pool: pg_pool });

    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info,sqlx=off,debug"));

    let api_prefix = "/api/v1";
    const BIND_ADDRESS: &str = "0.0.0.0:3030";
    let server = HttpServer::new(move || {
        let api_v1_scope =
            Scope::new(api_prefix)
                .configure(v1::configure_routes)
                .wrap(auth::middleware::Auth::<true> {
                    app_data: app_data.clone(),
                });

        App::new()
            .app_data(app_data.clone())
            .wrap(Logger::new("%a, '%r', %s, Bytes: %b %Dms"))
            .wrap(middleware::NormalizePath::new(
                middleware::TrailingSlash::Trim,
            ))
            .wrap(auth::middleware::Auth::<false> {
                app_data: app_data.clone(),
            })
            .service(api_v1_scope)
    })
    .bind(BIND_ADDRESS)?;
    eprintln!("Listening on {BIND_ADDRESS}");
    server.run().await?;

    Ok(())
}
