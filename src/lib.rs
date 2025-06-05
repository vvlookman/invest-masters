//! # invmst lib

use std::{path::PathBuf, sync::LazyLock};

use directories::ProjectDirs;

pub mod api;
pub mod error;

pub async fn init() {
    env_logger::Builder::new()
        .parse_filters(std::env::var("LOG").as_deref().unwrap_or("off"))
        .init();
}

static APP_DATA_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| match ProjectDirs::from("", "", env!("CARGO_PKG_NAME")) {
        Some(proj_dirs) => proj_dirs.data_dir().to_path_buf(),
        None => std::env::current_dir()
            .expect("Unable to get current directory!")
            .join("data"),
    });

mod data;
mod ds;
mod evaluate;
mod master;
mod ticker;
mod utils;
