use reqwest;
use serde::Serialize;

use crate::osm::Location;

#[derive(Serialize)]
pub struct SearchResult {
    label: String,
    location: Location,
    id: String,
}

pub async fn fuzzy_search(
    center: Location,
    query: &str,
) -> Result<Vec<SearchResult>, anyhow::Error> {
    let body = reqwest::get("https://www.rust-lang.org")
        .await?
        .text()
        .await?;
    println!("{body}");

    Ok(Vec::new())
}
