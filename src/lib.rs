//! # invmst lib

pub mod api;
pub mod error;

pub async fn init() {
    env_logger::Builder::new()
        .parse_filters(std::env::var("LOG").as_deref().unwrap_or("off"))
        .init();
}

mod evaluate;
mod masters;
