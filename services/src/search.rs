use reqwest;
use serde::{Deserialize, Serialize};
use std::env;

use crate::osm::Location;

#[derive(Serialize)]
pub struct SearchResult {
    id: String,
    label: String,
    location: Location,
    access: Option<Location>,
}

impl From<&HereDiscoverItem> for SearchResult {
    fn from(item: &HereDiscoverItem) -> Self {
        Self {
            id: item.id.clone(),
            label: item.title.clone(),
            location: item.position.clone().into(),
            access: item.access.first().map(|pos| pos.clone().into()),
        }
    }
}

impl From<HereDiscoverPosition> for Location {
    fn from(pos: HereDiscoverPosition) -> Self {
        Self {
            lat: pos.lat,
            lon: pos.lng,
        }
    }
}

#[derive(Clone, Deserialize, Debug)]
struct HereDiscoverPosition {
    lat: f64,
    lng: f64,
}

#[derive(Deserialize, Debug)]
struct HereDiscoverItem {
    id: String,
    title: String,
    position: HereDiscoverPosition,
    access: Vec<HereDiscoverPosition>,
}

#[derive(Deserialize, Debug)]
struct HereDiscoverResponse {
    items: Vec<HereDiscoverItem>,
}

pub async fn fuzzy_search(
    center: Location,
    query: &str,
) -> Result<Vec<SearchResult>, anyhow::Error> {
    let api_key = env::var("HERE_API_KEY")?;
    let formatted_query = query.replace(' ', "+");
    let url = format!(
        "https://discover.search.hereapi.com/v1/discover?at={},{}&limit=5&q={}&apiKey={}",
        center.lat, center.lon, formatted_query, api_key
    );

    let body = reqwest::get(&url)
        .await?
        .json::<HereDiscoverResponse>()
        .await?;

    Ok(body.items.iter().map(SearchResult::from).collect())
}
