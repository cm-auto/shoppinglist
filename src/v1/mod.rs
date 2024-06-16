const VERSION: &str = "v1";

macro_rules! resource_name {
    ($name: literal) => {
        &format!("{}{}", super::VERSION, $name)
    };
}

mod handlers;
mod models;
mod routes;
pub use routes::configure_routes;
