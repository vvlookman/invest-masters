use std::collections::HashMap;

use crate::{
    error::InvmstResult,
    utils::net::{http_get, join_url},
};

pub async fn call_public_api(
    path: &str,
    query: &HashMap<String, String>,
) -> InvmstResult<serde_json::Value> {
    let api_url = join_url(
        std::env::var("AKTOOLS_API")
            .as_deref()
            .unwrap_or("http://127.0.0.1:8080"),
        "/api/public",
    )?;

    let bytes = http_get(&api_url, Some(path), query, &HashMap::new()).await?;
    let json: serde_json::Value = serde_json::from_slice(&bytes)?;

    Ok(json)
}
